<script setup lang="ts">
/**
 * DesignStudio.vue - 设计工作室主页面
 * 
 * 这个组件是整个编辑器的核心，包含以下功能：
 * 1. Markdown 所见即所得编辑器（基于 Toast UI Editor）
 * 2. PlantUML 图表实时渲染
 * 3. AI 生成图表功能
 * 4. 文档管理（新建、保存、导出、加载）
 * 5. 模板库（内置模板 + 远程模板）
 * 6. AI 配置管理
 * 
 * 技术栈：Vue 3 + TypeScript + Toast UI Editor + Tauri
 */

import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import Editor from '@toast-ui/editor';
import '@toast-ui/editor/dist/toastui-editor.css';
import pako from 'pako';

// 导入子组件
import AIConfigModal, { type AIConfig } from '../components/AIConfigModal.vue';
import RemoteTemplates, { type RemoteTemplate } from '../components/RemoteTemplates.vue';
import TemplateUploadModal from '../components/TemplateUploadModal.vue';

// 导入图标组件（来自 lucide-vue-next 图标库）
import { 
  FileText,     // 文件图标
  Save,         // 保存图标
  Plus,         // 加号图标
  RefreshCw,    // 刷新图标
  Download,     // 下载图标
  Bot,          // 机器人图标
  Loader2,      // 加载动画图标
  Sparkles,     // 闪光图标
  FolderOpen,   // 文件夹图标
  Wand2,        // 魔法棒图标
  Type,         // 文字图标
  Eye,          // 眼睛图标
  LayoutTemplate, // 布局模板图标
  Settings,     // 设置图标
  BookOpen,     // 书本图标
  Upload        // 上传图标（新增）
} from 'lucide-vue-next';

// ==================== 响应式状态定义 ====================

/** 当前文件名 */
const currentFile = ref('untitled');

/** 文档内容（Markdown 格式） */
const docContent = ref('# 新文档\n\n在这里开始编写...');

/** 已保存的文件列表 */
const savedFiles = ref<string[]>([]);

/** 是否正在保存中 */
const isSaving = ref(false);

/** 是否正在生成图表中 */
const isGenerating = ref(false);

/** 编辑器容器 DOM 元素的引用 */
const editorContainer = ref<HTMLElement | null>(null);

/** Toast UI Editor 实例 */
const editor = ref<Editor | null>(null);

/** AI 输入框的内容 */
const aiInput = ref('');

/** 是否显示 AI 面板 */
const showAiPanel = ref(false);

/** 当前选中的文本（用于生成图表） */
const selectedText = ref('');

/** 是否显示 Toast 提示 */
const showToast = ref(false);

/** Toast 提示的消息内容 */
const toastMessage = ref('');

/** 是否显示 AI 配置弹窗 */
const showAIConfig = ref(false);

/** AI 配置对象 */
const aiConfig = ref<AIConfig>({
  provider: 'demo',      // AI 提供商：demo/openai/azure/custom
  apiKey: '',            // API 密钥
  apiUrl: '',            // API 地址
  model: 'gpt-4',        // 模型名称
  temperature: 0.7,      // 温度参数（随机性）
  maxTokens: 2000,       // 最大 Token 数
  timeout: 30,           // 超时时间（秒）
  templateUrl: 'https://api.example.com/templates',  // 远程模板地址
  enableRemoteTemplate: false,  // 是否启用远程模板
  enableStreaming: true,        // 是否启用流式输出
  enableAutoSuggest: false,     // 是否启用自动建议
});

/** 是否显示模板库弹窗 */
const showTemplateLibrary = ref(false);

/** 是否显示模板上传弹窗（新增） */
const showTemplateUpload = ref(false);

// ==================== 编辑器相关函数 ====================

/**
 * 初始化 Toast UI Editor 编辑器
 * 
 * 配置说明：
 * - initialEditType: 'wysiwyg' 表示所见即所得模式
 * - previewStyle: 'tab' 表示预览标签样式
 * - toolbarItems: 工具栏按钮配置
 * - customHTMLRenderer: 自定义 HTML 渲染器（用于渲染 PlantUML）
 */
const initEditor = () => {
  // 检查容器是否存在
  if (!editorContainer.value) return;
  
  // 如果已有编辑器实例，先销毁（避免重复创建）
  if (editor.value) {
    editor.value.destroy();
  }
  
  // 创建新的编辑器实例
  editor.value = new Editor({
    // 挂载的 DOM 元素
    el: editorContainer.value,
    // 编辑器高度
    height: '100%',
    // 初始编辑模式：wysiwyg = 所见即所得，markdown = 分屏预览
    initialEditType: 'wysiwyg',
    // 预览样式
    previewStyle: 'tab',
    // 初始内容
    initialValue: docContent.value,
    // 主题：light 或 dark
    theme: 'light',
    // 工具栏按钮配置（二维数组，每个子数组是一组按钮）
    toolbarItems: [
      ['heading', 'bold', 'italic', 'strike'],  // 标题、加粗、斜体、删除线
      ['hr', 'quote'],                           // 分割线、引用
      ['ul', 'ol', 'task', 'indent', 'outdent'], // 列表、任务列表、缩进
      ['table', 'image', 'link'],                // 表格、图片、链接
      ['code', 'codeblock'],                     // 行内代码、代码块
    ],
    // 自动聚焦
    autofocus: true,
    // 启用快捷键
    useCommandShortcut: true,
    // 占位符文本
    placeholder: '开始编写您的文档...',
    
    // 事件监听
    events: {
      // 内容变化时触发
      change: () => {
        if (editor.value) {
          // 同步更新 docContent
          docContent.value = editor.value.getMarkdown();
        }
      }
    },
    
    // 自定义 HTML 渲染器 - 关键功能：将 PlantUML 代码块渲染为图片
    customHTMLRenderer: {
      codeBlock(node: any) {
        const { language, content } = node;
        
        // 如果是 plantuml 代码块，渲染为图片
        if (language === 'plantuml') {
          // 1. 将 PlantUML 文本编码为 URL 安全的格式
          const encoded = encodePlantUML(content);
          // 2. 构建 PlantUML 在线服务的图片 URL
          const imageUrl = `https://www.plantuml.com/plantuml/svg/${encoded}`;
          
          // 返回自定义 HTML 结构
          return [
            // 外层容器
            { type: 'openTag', tagName: 'div', outerNewLine: true, attributes: { class: 'plantuml-wrapper' } },
            // 图片元素（如果加载失败显示 fallback）
            { type: 'html', content: `<img src="${imageUrl}" alt="PlantUML Diagram" class="plantuml-image" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';" />` },
            // Fallback 代码块（图片加载失败时显示）
            { type: 'html', content: `<pre class="plantuml-fallback" style="display:none;"><code class="language-plantuml">${escapeHtml(content)}</code></pre>` },
            // 关闭外层容器
            { type: 'closeTag', tagName: 'div', outerNewLine: true }
          ];
        }
        // 其他语言使用默认渲染
        return null;
      }
    }
  });
};

/**
 * PlantUML 编码函数
 * 
 * PlantUML 使用特殊的编码格式，需要：
 * 1. 先用 deflate 压缩文本
 * 2. 然后用自定义的 base64 变体编码
 * 
 * 这是 PlantUML 在线服务的标准编码方式
 */
const encodePlantUML = (text: string): string => {
  try {
    const data = new TextEncoder().encode(text);
    const compressed = pako.deflate(data, { level: 9 });
    return encode64(String.fromCharCode(...compressed));
  } catch (e) {
    return btoa(unescape(encodeURIComponent(text)));
  }
};

/**
 * 自定义 base64 编码（PlantUML 专用）
 * PlantUML 使用自己定义的 64 个字符，与标准 base64 不同
 */
const encode64 = (data: string): string => {
  let r = '';
  // 每 3 个字节一组进行编码
  for (let i = 0; i < data.length; i += 3) {
    if (i + 2 === data.length) {
      r += append3bytes(data.charCodeAt(i), data.charCodeAt(i + 1), 0);
    } else if (i + 1 === data.length) {
      r += append3bytes(data.charCodeAt(i), 0, 0);
    } else {
      r += append3bytes(data.charCodeAt(i), data.charCodeAt(i + 1), data.charCodeAt(i + 2));
    }
  }
  return r;
};

/**
 * 将 3 个字节编码为 4 个字符
 * PlantUML 的编码字符集：0-9A-Za-z-_
 */
const append3bytes = (b1: number, b2: number, b3: number): string => {
  const c1 = b1 >> 2;
  const c2 = ((b1 & 0x3) << 4) | (b2 >> 4);
  const c3 = ((b2 & 0xF) << 2) | (b3 >> 6);
  const c4 = b3 & 0x3F;
  const chars = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_';
  return chars.charAt(c1) + chars.charAt(c2) + chars.charAt(c3) + chars.charAt(c4);
};

/**
 * HTML 转义函数
 * 防止 XSS 攻击，将特殊字符转换为 HTML 实体
 */
const escapeHtml = (text: string): string => {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
};

/**
 * 获取编辑器中选中的文本
 * 
 * 实现方式：
 * 1. 首先尝试使用编辑器的 getSelectedText API
 * 2. 如果不可用，使用浏览器原生的 Selection API
 */
const getSelectedText = (): string => {
  if (!editor.value) return '';
  
  // 尝试使用编辑器的 API
  const editorInstance = (editor.value as any);
  if (editorInstance.getSelectedText) {
    return editorInstance.getSelectedText();
  }
  
  // 回退方案：使用浏览器 Selection API
  const selection = window.getSelection();
  if (selection && selection.toString()) {
    return selection.toString();
  }
  
  return '';
};

// ==================== 文档操作函数 ====================

/**
 * 创建新文档
 * 
 * 流程：
 * 1. 弹出确认框（防止误操作丢失内容）
 * 2. 生成新的文件名（带时间戳）
 * 3. 重置编辑器内容
 */
const createNewDocument = () => {
  if (confirm('创建新文档？当前内容将丢失')) {
    // 生成带时间戳的文件名，如：untitled_7843
    currentFile.value = 'untitled_' + Date.now().toString().slice(-4);
    const newContent = '# 新文档\n\n在这里开始编写...';
    docContent.value = newContent;
    
    // nextTick 确保 DOM 更新后再操作编辑器
    nextTick(() => {
      if (editor.value) {
        editor.value.setMarkdown(newContent);
      }
    });
  }
};

/**
 * 保存文档
 * 
 * 流程：
 * 1. 检查文件名是否为空
 * 2. 调用 Tauri 后端命令 save_document
 * 3. 刷新文件列表
 * 4. 显示成功提示
 */
const handleSave = async () => {
  // 如果文件名为空，提示用户输入
  if (!currentFile.value.trim()) {
    const name = prompt('请输入文件名:', 'document');
    if (!name) return;
    currentFile.value = name;
  }
  
  isSaving.value = true;
  try {
    // 调用 Rust 后端命令
    // invoke 是 Tauri 提供的 RPC 调用函数
    await invoke('save_document', { 
      filename: currentFile.value,
      content: docContent.value 
    });
    // 刷新文件列表
    await loadSavedFiles();
    showToastMessage('保存成功');
  } catch (err) {
    console.error(err);
    showToastMessage('保存失败: ' + err);
  } finally {
    isSaving.value = false;
  }
};

/**
 * 导出文档为 Markdown 文件
 * 
 * 原理：
 * 1. 创建 Blob 对象（二进制大对象）
 * 2. 生成临时 URL
 * 3. 创建隐藏的 a 标签触发下载
 * 4. 清理资源
 */
const exportDoc = () => {
  // 创建 Blob，指定 MIME 类型为 text/markdown
  const blob = new Blob([docContent.value], { type: 'text/markdown' });
  // 生成对象 URL
  const url = URL.createObjectURL(blob);
  
  // 创建临时 a 标签
  const a = document.createElement('a');
  a.href = url;
  // 设置下载文件名
  a.download = `${currentFile.value || 'document'}.md`;
  
  // 触发点击（不需要真正添加到 DOM）
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  
  // 释放 URL 对象（防止内存泄漏）
  URL.revokeObjectURL(url);
  showToastMessage('导出成功');
};

/**
 * 加载已保存的文件列表
 * 
 * 调用后端 list_documents 命令获取文档目录中的所有 .md 文件
 */
const loadSavedFiles = async () => {
  try {
    const files = await invoke('list_documents') as string[];
    savedFiles.value = files;
  } catch (err) {
    console.error('加载失败:', err);
  }
};

/**
 * 加载指定文档
 * 
 * @param filename 文件名
 */
const loadDocument = async (filename: string) => {
  try {
    // 调用后端加载文档内容
    const content = await invoke('load_document', { filename }) as string;
    currentFile.value = filename;
    docContent.value = content;
    
    nextTick(() => {
      if (editor.value) {
        editor.value.setMarkdown(content);
      }
    });
    showToastMessage(`已加载: ${filename}`);
  } catch (err) {
    showToastMessage('加载失败: ' + err);
  }
};

// ==================== AI 配置相关 ====================

/** 打开 AI 配置弹窗 */
const openAIConfig = () => {
  showAIConfig.value = true;
};

/**
 * 保存 AI 配置
 * 
 * @param config AI 配置对象
 */
const saveAIConfig = (config: AIConfig) => {
  aiConfig.value = config;
};

// ==================== 模板库相关 ====================

/** 打开模板库弹窗 */
const openTemplateLibrary = () => {
  showTemplateLibrary.value = true;
};

/**
 * 使用选中的模板
 * 
 * @param template 模板对象
 */
const useTemplate = (template: RemoteTemplate) => {
  if (editor.value) {
    const currentContent = editor.value.getMarkdown();
    // 在文档末尾追加模板内容
    const newContent = currentContent + '\n\n' + template.content;
    editor.value.setMarkdown(newContent);
    showToastMessage(`已应用模板: ${template.name}`);
  }
};

// ==================== 模板上传功能（新增）====================

/**
 * 打开模板上传弹窗
 * 将当前编辑器内容作为模板上传
 */
const openTemplateUpload = () => {
  if (!editor.value || !docContent.value.trim()) {
    showToastMessage('请先编写一些内容再上传');
    return;
  }
  showTemplateUpload.value = true;
};

/**
 * 处理模板上传成功
 */
const handleTemplateUploaded = () => {
  showToastMessage('模板上传成功！');
};

// ==================== AI 生成 PlantUML 功能 ====================

/**
 * 根据选中文本生成 PlantUML 图表
 * 
 * 流程：
 * 1. 获取编辑器中选中的文本
 * 2. 检查是否配置了真实 AI（非 demo 模式）
 * 3. 调用后端接口生成 PlantUML 代码
 * 4. 将生成的代码插入到文档中
 */
const generatePlantUMLFromSelection = async () => {
  // 1. 获取选中的文本
  const text = getSelectedText();
  if (!text.trim()) {
    showToastMessage('请先选中一段文字描述');
    return;
  }
  
  selectedText.value = text;
  isGenerating.value = true;
  
  try {
    // 2. 检查 AI 配置
    if (aiConfig.value.provider !== 'demo' && aiConfig.value.apiKey) {
      // 真实 AI 模式：调用后端接口
      const result = await invoke('generate_plantuml_from_text', { 
        description: text 
      }) as string;
      insertPlantUMLToEditor(result);
      showToastMessage('PlantUML 图表已生成');
    } else {
      // 演示模式：使用本地规则生成
      const demoPlantUML = generateDemoPlantUML(text);
      insertPlantUMLToEditor(demoPlantUML);
      showToastMessage('已生成示例图表（演示模式）');
    }
  } catch (err) {
    console.error(err);
    // 出错时回退到演示模式
    const demoPlantUML = generateDemoPlantUML(text);
    insertPlantUMLToEditor(demoPlantUML);
    showToastMessage('已生成示例图表（演示模式）');
  } finally {
    isGenerating.value = false;
  }
};

/**
 * 演示模式：根据关键词生成示例 PlantUML
 * 
 * 这是在没有配置真实 AI 时的回退方案
 * 通过关键词匹配生成不同类型的图表
 * 
 * @param text 用户输入的文本
 * @returns PlantUML 代码
 */
const generateDemoPlantUML = (text: string): string => {
  const desc_lower = text.toLowerCase();
  
  // 根据关键词判断图表类型
  if (desc_lower.includes('登录') || desc_lower.includes('认证') || desc_lower.includes('鉴权')) {
    return `\`\`\`plantuml
@startuml
!theme plain
skinparam backgroundColor #FEFEFE
title ${text}

actor 用户 as User
participant "前端应用" as Frontend
participant "认证服务\\nAuth Service" as Auth
database "用户数据" as DB

User -> Frontend: 1. 输入账号密码
Frontend -> Auth: 2. POST /login
Auth -> DB: 3. 查询用户信息
DB --> Auth: 4. 返回用户数据
Auth -> Auth: 5. 验证密码哈希

alt 验证成功
    Auth --> Frontend: 6a. 返回 JWT Token
    Frontend --> User: 7a. 登录成功跳转
else 验证失败
    Auth --> Frontend: 6b. 返回 401 错误
    Frontend --> User: 7b. 显示错误提示
end

@enduml
\`\`\``;
  } else if (desc_lower.includes('订单') || desc_lower.includes('下单') || desc_lower.includes('购买')) {
    return `\`\`\`plantuml
@startuml
!theme plain
skinparam backgroundColor #FEFEFE
title ${text}

actor 用户 as User
participant "订单服务\\nOrder Service" as Order
participant "库存服务\\nStock Service" as Stock
participant "支付服务\\nPayment Service" as Pay
database "订单数据库" as OrderDB

group 创建订单
    User -> Order: 1. 提交订单请求
    Order -> Stock: 2. 检查库存
    Stock --> Order: 3. 库存充足
    Order -> OrderDB: 4. 保存订单(状态:待支付)
    Order --> User: 5. 订单创建成功
end

group 支付流程
    User -> Pay: 6. 发起支付
    Pay -> OrderDB: 7. 更新订单状态(已支付)
    OrderDB --> Pay: 8. 更新成功
    Pay --> User: 9. 支付成功确认
end

@enduml
\`\`\``;
  } else if (desc_lower.includes('类') || desc_lower.includes('class') || desc_lower.includes('对象')) {
    return `\`\`\`plantuml
@startuml
!theme plain
skinparam backgroundColor #FEFEFE
title ${text} - 类图

class User {
  - Long id
  - String username
  - String email
  + Boolean login()
  + void logout()
}

class Order {
  - Long id
  - Long userId
  - BigDecimal amount
  + Boolean pay()
  + Boolean cancel()
}

User "1" --> "*" Order : 拥有 >

@enduml
\`\`\``;
  } else {
    // 默认流程图
    return `\`\`\`plantuml
@startuml
!theme plain
skinparam backgroundColor #FEFEFE
title ${text}

start
:接收用户输入;
:系统处理请求;
if (条件判断?) then (条件成立)
  :执行分支 A;
else (条件不成立)
  :执行分支 B;
endif
:返回响应;
stop

@enduml
\`\`\``;
  }
};

/**
 * 将 PlantUML 代码插入到编辑器
 * 
 * @param plantumlCode PlantUML 代码
 */
const insertPlantUMLToEditor = (plantumlCode: string) => {
  if (!editor.value) return;
  
  const currentContent = editor.value.getMarkdown();
  // 在文档末尾追加，添加标题和代码块
  const newContent = currentContent + '\n\n## 自动生成图表\n\n' + plantumlCode;
  
  docContent.value = newContent;
  editor.value.setMarkdown(newContent);
};

/**
 * 根据 AI 输入框内容生成 PlantUML
 */
const generatePlantUMLFromInput = async () => {
  if (!aiInput.value.trim()) {
    showToastMessage('请输入需求描述');
    return;
  }
  
  isGenerating.value = true;
  try {
    // 调用后端接口
    const result = await invoke('generate_plantuml', { 
      description: aiInput.value 
    }) as string;
    
    insertPlantUMLToEditor(result);
    aiInput.value = '';
    showAiPanel.value = false;
    showToastMessage('PlantUML 图表已生成');
  } catch (err) {
    console.error(err);
    // 演示模式回退
    const demoPlantUML = generateDemoPlantUML(aiInput.value);
    insertPlantUMLToEditor(demoPlantUML);
    aiInput.value = '';
    showAiPanel.value = false;
    showToastMessage('已生成示例图表（演示模式）');
  } finally {
    isGenerating.value = false;
  }
};

/**
 * 插入预设的 PlantUML 模板
 * 
 * @param type 模板类型：uml / class / flow
 */
const insertTemplate = (type: string) => {
  if (!editor.value) return;
  
  let template = '';
  switch (type) {
    case 'uml':
      template = `\n\n## 时序图\n\n\`\`\`plantuml
@startuml
!theme plain
actor 用户
participant "系统" as System
database "数据库" as DB

用户 -> System: 请求
System -> DB: 查询
DB --> System: 返回数据
System --> 用户: 响应
@enduml
\`\`\`\n`;
      break;
    case 'class':
      template = `\n\n## 类图\n\n\`\`\`plantuml
@startuml
!theme plain
class ClassName {
  - attribute: Type
  + method(): ReturnType
}
ClassName --|> ParentClass : 继承
@enduml
\`\`\`\n`;
      break;
    case 'flow':
      template = `\n\n## 流程图\n\n\`\`\`plantuml
@startuml
!theme plain
start
:开始;
if (条件?) then (是)
  :操作1;
else (否)
  :操作2;
endif
:结束;
stop
@enduml
\`\`\`\n`;
      break;
  }
  
  const currentContent = editor.value.getMarkdown();
  editor.value.setMarkdown(currentContent + template);
  showToastMessage('模板已插入');
};

// ==================== 工具函数 ====================

/**
 * 显示 Toast 提示
 * 
 * @param msg 消息内容
 */
const showToastMessage = (msg: string) => {
  toastMessage.value = msg;
  showToast.value = true;
  // 3 秒后自动隐藏
  setTimeout(() => {
    showToast.value = false;
  }, 3000);
};

// ==================== 生命周期钩子 ====================

/**
 * 组件挂载时执行
 * 
 * 初始化流程：
 * 1. 加载文件列表
 * 2. 初始化编辑器
 * 3. 加载保存的 AI 配置
 */
onMounted(() => {
  loadSavedFiles();
  nextTick(() => {
    initEditor();
  });
  
  // 从 localStorage 加载 AI 配置
  const savedConfig = localStorage.getItem('ai-flow-studio-ai-config');
  if (savedConfig) {
    try {
      aiConfig.value = JSON.parse(savedConfig);
    } catch (e) {
      console.error('加载 AI 配置失败:', e);
    }
  }
});

/**
 * 组件卸载前执行
 * 
 * 清理工作：销毁编辑器实例，防止内存泄漏
 */
onBeforeUnmount(() => {
  if (editor.value) {
    editor.value.destroy();
    editor.value = null;
  }
});
</script>

<template>
  <!-- 主容器：flex 布局，左右结构 -->
  <div class="flex h-full bg-gray-50 text-gray-800 font-sans overflow-hidden">
    
    <!-- ==================== 左侧边栏 ==================== -->
    <aside class="w-64 bg-white border-r border-gray-200 flex flex-col flex-shrink-0 shadow-sm">
      
      <!-- Logo 区域 -->
      <div class="p-4 border-b border-gray-200">
        <div class="flex items-center gap-2">
          <!-- 渐变色背景图标 -->
          <div class="bg-gradient-to-br from-blue-500 to-purple-600 p-2 rounded-lg">
            <Sparkles :size="18" class="text-white" />
          </div>
          <div>
            <h1 class="font-bold text-gray-800">设计工作室</h1>
            <p class="text-xs text-gray-500">AI 辅助设计</p>
          </div>
        </div>
      </div>

      <!-- 快捷操作按钮组：新建、保存、导出 -->
      <div class="p-3 grid grid-cols-3 gap-2 border-b border-gray-200">
        <button 
          @click="createNewDocument"
          class="flex flex-col items-center gap-1 p-2 rounded-lg hover:bg-gray-100 transition-colors"
          title="新建文档"
        >
          <Plus :size="18" class="text-gray-600" />
          <span class="text-[10px] text-gray-600">新建</span>
        </button>
        
        <button 
          @click="handleSave"
          :disabled="isSaving"
          class="flex flex-col items-center gap-1 p-2 rounded-lg hover:bg-gray-100 transition-colors disabled:opacity-50"
          title="保存文档"
        >
          <Save :size="18" class="text-blue-600" />
          <span class="text-[10px] text-gray-600">保存</span>
        </button>
        
        <button 
          @click="exportDoc"
          class="flex flex-col items-center gap-1 p-2 rounded-lg hover:bg-gray-100 transition-colors"
          title="导出 Markdown"
        >
          <Download :size="18" class="text-green-600" />
          <span class="text-[10px] text-gray-600">导出</span>
        </button>
      </div>

      <!-- 模板库按钮 -->
      <div class="p-3 border-b border-gray-200">
        <button 
          @click="openTemplateLibrary"
          class="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg bg-gradient-to-r from-blue-500 to-indigo-500 text-white hover:opacity-90 transition-all"
        >
          <BookOpen :size="16" />
          <span class="text-sm font-medium">模板库</span>
        </button>
      </div>

      <!-- 快速插入 PlantUML 模板 -->
      <div class="p-3 border-b border-gray-200">
        <p class="text-xs font-medium text-gray-500 mb-2 uppercase tracking-wider">快速插入</p>
        <div class="grid grid-cols-3 gap-2">
          <button 
            @click="insertTemplate('uml')"
            class="flex flex-col items-center gap-1 p-2 rounded-lg bg-blue-50 hover:bg-blue-100 transition-colors"
            title="插入时序图模板"
          >
            <Bot :size="16" class="text-blue-600" />
            <span class="text-[10px] text-blue-700">时序图</span>
          </button>
          <button 
            @click="insertTemplate('class')"
            class="flex flex-col items-center gap-1 p-2 rounded-lg bg-purple-50 hover:bg-purple-100 transition-colors"
            title="插入类图模板"
          >
            <LayoutTemplate :size="16" class="text-purple-600" />
            <span class="text-[10px] text-purple-700">类图</span>
          </button>
          <button 
            @click="insertTemplate('flow')"
            class="flex flex-col items-center gap-1 p-2 rounded-lg bg-green-50 hover:bg-green-100 transition-colors"
            title="插入流程图模板"
          >
            <Eye :size="16" class="text-green-600" />
            <span class="text-[10px] text-green-700">流程图</span>
          </button>
        </div>
      </div>

      <!-- AI 生成图表面板 -->
      <div class="p-3 border-b border-gray-200">
        <!-- 展开/收起按钮 -->
        <button 
          @click="showAiPanel = !showAiPanel"
          :class="[
            'w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg transition-all',
            showAiPanel 
              ? 'bg-gradient-to-r from-purple-600 to-pink-600 text-white' 
              : 'bg-gradient-to-r from-purple-500 to-pink-500 text-white hover:opacity-90'
          ]"
        >
          <Wand2 :size="16" />
          <span class="text-sm font-medium">AI 生成图表</span>
        </button>
        
        <!-- AI 输入面板（条件渲染） -->
        <div v-if="showAiPanel" class="mt-2 space-y-2 animate-fade-in">
          <textarea 
            v-model="aiInput"
            placeholder="描述你想要的图表，例如：用户登录流程图..."
            class="w-full h-20 p-2 text-xs border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-purple-500"
          />
          <button 
            @click="generatePlantUMLFromInput"
            :disabled="isGenerating"
            class="w-full py-1.5 px-3 rounded-lg bg-purple-600 text-white text-xs font-medium hover:bg-purple-700 disabled:opacity-50 flex items-center justify-center gap-1"
          >
            <Loader2 v-if="isGenerating" :size="12" class="animate-spin" />
            <span>{{ isGenerating ? '生成中...' : '生成 PlantUML' }}</span>
          </button>
        </div>
      </div>

      <!-- 文件列表 -->
      <div class="flex-1 overflow-hidden flex flex-col">
        <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200">
          <span class="text-xs font-medium text-gray-500">文档列表</span>
          <button @click="loadSavedFiles" class="p-1 hover:bg-gray-100 rounded">
            <RefreshCw :size="12" class="text-gray-400" />
          </button>
        </div>
        <div class="flex-1 overflow-y-auto p-2 space-y-1">
          <!-- 遍历显示文件列表 -->
          <button 
            v-for="file in savedFiles" 
            :key="file"
            @click="loadDocument(file)"
            :class="[
              'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-sm transition-colors',
              currentFile === file 
                ? 'bg-blue-50 text-blue-700 border border-blue-200' 
                : 'hover:bg-gray-100 text-gray-700'
            ]"
          >
            <FileText :size="14" :class="currentFile === file ? 'text-blue-500' : 'text-gray-400'" />
            <span class="truncate">{{ file }}</span>
          </button>
          <!-- 空状态提示 -->
          <div v-if="savedFiles.length === 0" class="text-center py-4 text-xs text-gray-400">
            暂无文档
          </div>
        </div>
      </div>

      <!-- 底部操作区：AI 配置 + 模板上传 -->
      <div class="p-3 border-t border-gray-200 space-y-2">
        <!-- AI 配置按钮 -->
        <button 
          @click="openAIConfig"
          class="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg bg-gray-100 hover:bg-gray-200 text-gray-700 transition-all"
        >
          <Settings :size="16" />
          <span class="text-sm font-medium">AI 配置</span>
          <!-- 状态标签 -->
          <span 
            :class="[
              'ml-auto text-[10px] px-1.5 py-0.5 rounded-full',
              aiConfig.provider === 'demo' ? 'bg-gray-300 text-gray-600' : 'bg-green-100 text-green-700'
            ]"
          >
            {{ aiConfig.provider === 'demo' ? '演示' : '已配置' }}
          </span>
        </button>
        
        <!-- 模板上传按钮（新增） -->
        <button 
          @click="openTemplateUpload"
          class="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg bg-indigo-50 hover:bg-indigo-100 text-indigo-700 transition-all"
        >
          <Upload :size="16" />
          <span class="text-sm font-medium">上传为模板</span>
        </button>
      </div>
    </aside>

    <!-- ==================== 主编辑区 ==================== -->
    <main class="flex-1 flex flex-col bg-white min-w-0">
      
      <!-- 顶部工具栏 -->
      <header class="h-14 border-b border-gray-200 flex items-center justify-between px-4 bg-white">
        <!-- 左侧：文件名输入 -->
        <div class="flex items-center gap-3">
          <FolderOpen :size="18" class="text-gray-400" />
          <input 
            v-model="currentFile"
            type="text"
            placeholder="输入文件名"
            class="px-3 py-1.5 text-sm border border-gray-300 rounded-lg hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 w-48"
          />
          <span class="text-xs text-gray-400">.md</span>
        </div>
        
        <!-- 右侧：功能按钮 -->
        <div class="flex items-center gap-2">
          <!-- AI 生成图表按钮 -->
          <button
            @click="generatePlantUMLFromSelection"
            :disabled="isGenerating"
            class="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-indigo-500 to-purple-500 text-white text-sm font-medium hover:opacity-90 disabled:opacity-50 transition-all shadow-sm"
            title="选中文字后点击，AI 自动生成 PlantUML 图表"
          >
            <Loader2 v-if="isGenerating" :size="16" class="animate-spin" />
            <Wand2 v-else :size="16" />
            <span>选中文本生成图表</span>
          </button>
          
          <!-- 分隔线 -->
          <div class="w-px h-6 bg-gray-200 mx-1"></div>
          
          <!-- 字数统计 -->
          <div class="flex items-center gap-2 px-3 py-1.5 bg-gray-50 rounded-lg">
            <Type :size="14" class="text-gray-400" />
            <span class="text-xs text-gray-500">{{ docContent.length }} 字符</span>
          </div>
        </div>
      </header>

      <!-- 编辑器容器 -->
      <div class="flex-1 overflow-hidden p-4 bg-gray-50">
        <!-- ref="editorContainer" 用于挂载 Toast UI Editor -->
        <div ref="editorContainer" class="h-full rounded-xl border border-gray-200 overflow-hidden bg-white shadow-sm" />
      </div>
    </main>

    <!-- ==================== 全局组件 ==================== -->
    
    <!-- Toast 提示组件（固定在底部中央） -->
    <transition name="toast">
      <div 
        v-if="showToast" 
        class="fixed bottom-6 left-1/2 -translate-x-1/2 px-6 py-3 bg-gray-800 text-white text-sm rounded-lg shadow-lg z-50 flex items-center gap-2"
      >
        <Sparkles :size="16" class="text-yellow-400" />
        {{ toastMessage }}
      </div>
    </transition>

    <!-- AI 配置弹窗（Teleport 到 body，避免 z-index 问题） -->
    <AIConfigModal
      v-model:show="showAIConfig"
      @close="showAIConfig = false"
      @save="saveAIConfig"
    />

    <!-- 模板库弹窗 -->
    <RemoteTemplates
      :show="showTemplateLibrary"
      :template-url="aiConfig.templateUrl"
      @close="showTemplateLibrary = false"
      @select="useTemplate"
    />
    
    <!-- 模板上传弹窗（新增） -->
    <TemplateUploadModal
      :show="showTemplateUpload"
      :content="docContent"
      @close="showTemplateUpload = false"
      @uploaded="handleTemplateUploaded"
    />
  </div>
</template>

<style scoped>
/**
 * 自定义样式说明：
 * 
 * 1. :deep() - Vue 3 的深度选择器，用于穿透组件作用域修改子组件样式
 * 2. 所有 Toast UI Editor 的样式都在 :deep() 中定义
 * 3. 自定义动画使用 @keyframes 定义
 */

/* Toast UI Editor 基础样式覆盖 */
:deep(.toastui-editor-defaultUI) {
  border: none;
}

:deep(.toastui-editor-toolbar) {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  padding: 4px 8px;
}

:deep(.toastui-editor-toolbar button) {
  color: #4b5563;
  border-radius: 6px;
  margin: 0 2px;
}

:deep(.toastui-editor-toolbar button:hover) {
  background: #e5e7eb;
}

:deep(.toastui-editor-main) {
  background: #ffffff;
}

/* 编辑器内容区域 */
:deep(.ProseMirror) {
  padding: 1.5rem;
  color: #1f2937;
}

:deep(.toastui-editor-contents) {
  font-size: 15px;
  line-height: 1.7;
}

/* 标题样式 */
:deep(.toastui-editor-contents h1) {
  font-size: 1.75rem;
  font-weight: 700;
  margin: 1.5rem 0 0.75rem;
  color: #111827;
  border-bottom: 2px solid #e5e7eb;
  padding-bottom: 0.5rem;
}

:deep(.toastui-editor-contents h2) {
  font-size: 1.4rem;
  font-weight: 600;
  margin: 1.25rem 0 0.5rem;
  color: #1f2937;
}

:deep(.toastui-editor-contents h3) {
  font-size: 1.2rem;
  font-weight: 600;
  margin: 1rem 0 0.5rem;
  color: #1f2937;
}

/* 段落和列表 */
:deep(.toastui-editor-contents p) {
  margin: 0.75rem 0;
  color: #374151;
}

:deep(.toastui-editor-contents ul),
:deep(.toastui-editor-contents ol) {
  margin: 0.75rem 0;
  padding-left: 1.75rem;
}

:deep(.toastui-editor-contents li) {
  margin: 0.375rem 0;
}

/* 行内代码 */
:deep(.toastui-editor-contents code) {
  background: #f3f4f6;
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 0.875em;
  color: #dc2626;
}

/* 代码块 */
:deep(.toastui-editor-contents pre) {
  background: #1f2937;
  color: #f9fafb;
  padding: 1.25rem;
  border-radius: 0.75rem;
  overflow-x: auto;
  margin: 1rem 0;
  font-family: 'Fira Code', 'Consolas', monospace;
}

:deep(.toastui-editor-contents pre code) {
  background: transparent;
  color: inherit;
  padding: 0;
  font-size: 0.875rem;
}

/* 引用块 */
:deep(.toastui-editor-contents blockquote) {
  border-left: 4px solid #3b82f6;
  padding-left: 1rem;
  margin: 1rem 0;
  color: #6b7280;
  font-style: italic;
  background: #eff6ff;
  padding: 1rem 1rem 1rem 1.25rem;
  border-radius: 0 0.5rem 0.5rem 0;
}

/* 链接 */
:deep(.toastui-editor-contents a) {
  color: #3b82f6;
  text-decoration: none;
  border-bottom: 1px solid transparent;
  transition: border-color 0.2s;
}

:deep(.toastui-editor-contents a:hover) {
  border-bottom-color: #3b82f6;
}

/* 表格 */
:deep(.toastui-editor-contents table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1rem 0;
  border-radius: 0.5rem;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

:deep(.toastui-editor-contents th),
:deep(.toastui-editor-contents td) {
  border: 1px solid #e5e7eb;
  padding: 0.75rem;
  text-align: left;
}

:deep(.toastui-editor-contents th) {
  background: #f9fafb;
  font-weight: 600;
  color: #374151;
}

:deep(.toastui-editor-contents tr:nth-child(even)) {
  background: #f9fafb;
}

/* 图片 */
:deep(.toastui-editor-contents img) {
  max-width: 100%;
  height: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
}

/**
 * PlantUML 图表专用样式
 * 这些样式会应用到通过 customHTMLRenderer 渲染的 PlantUML 代码块
 */
:deep(.plantuml-wrapper) {
  margin: 1.5rem 0;
  padding: 1.5rem;
  background: #fafafa;
  border-radius: 0.75rem;
  border: 1px solid #e5e7eb;
  display: flex;
  justify-content: center;
  overflow-x: auto;
}

:deep(.plantuml-image) {
  max-width: 100%;
  height: auto;
  border-radius: 0.5rem;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

:deep(.plantuml-fallback) {
  margin: 0;
}

/* 淡入动画 */
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-5px); }
  to { opacity: 1; transform: translateY(0); }
}

.animate-fade-in {
  animation: fadeIn 0.2s ease-out;
}

/* Toast 动画 */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translate(-50%, 20px);
}

.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, -20px);
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #d1d5db;
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}
</style>
