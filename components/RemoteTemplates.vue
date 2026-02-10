<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  Download, 
  Globe, 
  RefreshCw, 
  Search,
  FileText,
  Check,
  AlertCircle,
  X,
  FolderOpen,
  Sparkles
} from 'lucide-vue-next';

export interface RemoteTemplate {
  id: string;
  name: string;
  description: string;
  content: string;
  category?: string;
  tags?: string[];
  author?: string;
  updatedAt?: string;
}

const props = defineProps<{
  show: boolean;
  templateUrl: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select', template: RemoteTemplate): void;
}>();

// 状态
const templates = ref<RemoteTemplate[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const searchQuery = ref('');
const selectedCategory = ref('all');
const selectedTemplate = ref<RemoteTemplate | null>(null);
const previewMode = ref(false);

// 内置演示模板
const demoTemplates: RemoteTemplate[] = [
  {
    id: 'demo-api-design',
    name: 'API 设计规范',
    description: 'RESTful API 设计规范模板，包含 URL 设计、请求方法、状态码等',
    category: '规范',
    tags: ['API', 'REST', '规范'],
    author: '系统',
    content: `# API 设计规范

## 1. URL 设计

### 基本规则
- 使用名词复数形式表示资源
- 使用小写字母，多个单词用连字符分隔
- 不使用动词，使用 HTTP 方法表示操作

### 示例
\`\`\`
GET    /api/v1/users          # 获取用户列表
GET    /api/v1/users/{id}     # 获取单个用户
POST   /api/v1/users          # 创建用户
PUT    /api/v1/users/{id}     # 更新用户
DELETE /api/v1/users/{id}     # 删除用户
\`\`\`

## 2. HTTP 方法

| 方法 | 用途 | 幂等性 |
|------|------|--------|
| GET | 获取资源 | 是 |
| POST | 创建资源 | 否 |
| PUT | 更新资源（完整） | 是 |
| PATCH | 更新资源（部分） | 否 |
| DELETE | 删除资源 | 是 |

## 3. 状态码

- \`200 OK\` - 请求成功
- \`201 Created\` - 创建成功
- \`400 Bad Request\` - 请求参数错误
- \`401 Unauthorized\` - 未认证
- \`403 Forbidden\` - 无权限
- \`404 Not Found\` - 资源不存在
- \`500 Internal Server Error\` - 服务器内部错误

## 4. 响应格式

\`\`\`json
{
  "code": 200,
  "message": "success",
  "data": { },
  "timestamp": 1704067200000
}
\`\`\`

## 5. PlantUML 示例

\`\`\`plantuml
@startuml
!theme plain
actor 客户端 as Client
participant "API Gateway" as Gateway
participant "用户服务" as UserService
database "MySQL" as DB

Client -> Gateway: GET /api/v1/users
Gateway -> UserService: 转发请求
UserService -> DB: SELECT * FROM users
DB --> UserService: 返回数据
UserService --> Gateway: JSON 响应
Gateway --> Client: 200 OK

@enduml
\`\`\`
`,
  },
  {
    id: 'demo-db-design',
    name: '数据库设计规范',
    description: '数据库表结构设计规范，包含命名规则、字段类型、索引设计等',
    category: '规范',
    tags: ['数据库', 'MySQL', '规范'],
    author: '系统',
    content: `# 数据库设计规范

## 1. 命名规范

### 表名
- 小写字母，下划线分隔
- 复数形式
- 业务前缀（可选）

示例：
\`\`\`
users              # 用户表
user_profiles      # 用户资料表
order_items        # 订单项表
\`\`\`

### 字段名
- 小写字母，下划线分隔
- 简洁明了

示例：
\`\`\`
id                 # 主键
created_at         # 创建时间
updated_at         # 更新时间
is_deleted         # 软删除标记
\`\`\`

## 2. 字段类型选择

| 类型 | 适用场景 | 示例 |
|------|----------|------|
| BIGINT | 主键、大数据量ID | user_id |
| INT | 状态、数量 | status, count |
| VARCHAR | 字符串（可变长） | username, email |
| CHAR | 固定长度 | phone, id_card |
| TEXT | 长文本 | content, description |
| DECIMAL | 精确小数 | amount, price |
| DATETIME | 日期时间 | created_at |
| TIMESTAMP | 时间戳 | updated_at |
| BOOLEAN/TINYINT | 布尔值 | is_active |

## 3. 索引设计

### 命名规范
\`\`\`
主键：pk_{table_name}
唯一：uk_{table_name}_{column}
普通：idx_{table_name}_{column}
联合：idx_{table_name}_{col1}_{col2}
\`\`\`

### 示例
\`\`\`sql
-- 用户表
CREATE TABLE users (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    status TINYINT DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME ON UPDATE CURRENT_TIMESTAMP,
    
    UNIQUE KEY uk_users_email (email),
    UNIQUE KEY uk_users_phone (phone),
    INDEX idx_users_status_created (status, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
\`\`\`

## 4. PlantUML 类图

\`\`\`plantuml
@startuml
!theme plain

class users {
  + BIGINT id
  + VARCHAR(50) username
  + VARCHAR(100) email
  + VARCHAR(20) phone
  + TINYINT status
  + DATETIME created_at
  + DATETIME updated_at
  --
  + login()
  + updateProfile()
}

class orders {
  + BIGINT id
  + BIGINT user_id
  + DECIMAL(10,2) total_amount
  + TINYINT status
  + DATETIME created_at
  --
  + create()
  + pay()
  + cancel()
}

users "1" -- "*" orders : 拥有 >

@enduml
\`\`\`
`,
  },
  {
    id: 'demo-prd-template',
    name: 'PRD 产品需求文档',
    description: '标准的产品需求文档模板，包含背景、需求描述、验收标准等',
    category: '文档',
    tags: ['PRD', '产品', '需求'],
    author: '系统',
    content: `# PRD - [功能名称]

## 1. 文档信息

| 项目 | 内容 |
|------|------|
| 文档版本 | v1.0 |
| 创建日期 | 2024-01-01 |
| 作者 | [姓名] |
| 审核人 | [姓名] |

## 2. 背景与目标

### 2.1 背景
描述当前的问题或机会点...

### 2.2 目标
- 目标1：...
- 目标2：...

### 2.3 预期收益
- 收益1：...
- 收益2：...

## 3. 需求范围

### 3.1 涉及范围
- [x] 功能A
- [ ] 功能B（二期）

### 3.2 用户群体
- 用户群体1：...
- 用户群体2：...

## 4. 功能需求

### 4.1 功能1：xxx

#### 需求描述
详细描述该功能...

#### 业务流程

\`\`\`plantuml
@startuml
!theme plain
start
:用户操作;
:系统处理;
if (条件?) then (是)
  :执行A;
else (否)
  :执行B;
endif
:结束;
stop
@enduml
\`\`\`

#### 页面原型
[原型图或描述]

#### 交互说明
1. 步骤1：...
2. 步骤2：...

## 5. 非功能需求

### 5.1 性能要求
- 页面加载时间 < 2s
- 接口响应时间 < 500ms

### 5.2 安全要求
- 数据加密传输
- 敏感操作需二次确认

## 6. 验收标准

- [ ] 标准1：...
- [ ] 标准2：...
- [ ] 标准3：...

## 7. 风险与依赖

| 风险 | 影响 | 应对措施 |
|------|------|----------|
| 风险1 | 高 | 措施1 |
| 风险2 | 中 | 措施2 |
`,
  },
  {
    id: 'demo-architecture',
    name: '系统架构设计文档',
    description: '微服务架构设计文档模板，包含架构图、技术选型、部署方案等',
    category: '架构',
    tags: ['架构', '微服务', '设计'],
    author: '系统',
    content: `# 系统架构设计文档

## 1. 概述

### 1.1 设计目标
- 高可用性
- 可扩展性
- 可维护性

### 1.2 技术栈

| 层级 | 技术选型 |
|------|----------|
| 前端 | Vue 3 + TypeScript |
| 后端 | Spring Boot / Go |
| 数据库 | MySQL + Redis |
| 消息队列 | RabbitMQ / Kafka |
| 容器化 | Docker + Kubernetes |

## 2. 系统架构

### 2.1 整体架构图

\`\`\`plantuml
@startuml
!theme plain

node "客户端层" {
  [Web App\nVue3] as WebApp
  [Mobile App\nFlutter] as MobileApp
}

node "接入层" {
  [Nginx] as Nginx
  [API Gateway] as Gateway
}

node "服务层" {
  package "微服务" {
    [用户服务] as UserService
    [订单服务] as OrderService
    [商品服务] as ProductService
    [支付服务] as PaymentService
  }
}

node "数据层" {
  database "MySQL" as MySQL
  database "Redis" as Redis
  database "Elasticsearch" as ES
}

WebApp --> Nginx
MobileApp --> Nginx
Nginx --> Gateway
Gateway --> UserService
Gateway --> OrderService
Gateway --> ProductService
Gateway --> PaymentService

UserService --> MySQL
OrderService --> MySQL
ProductService --> MySQL
UserService --> Redis
OrderService --> Redis
ProductService --> ES

@enduml
\`\`\`

### 2.2 服务划分

| 服务 | 职责 | 端口 |
|------|------|------|
| user-service | 用户管理 | 8001 |
| order-service | 订单管理 | 8002 |
| product-service | 商品管理 | 8003 |
| payment-service | 支付处理 | 8004 |

## 3. 核心流程

### 3.1 用户注册登录流程

\`\`\`plantuml
@startuml
!theme plain

actor 用户 as User
participant "前端" as FE
participant "API Gateway" as Gateway
participant "用户服务" as UserService
database "MySQL" as DB
database "Redis" as Redis

== 注册 ==
User -> FE: 填写注册信息
FE -> Gateway: POST /api/v1/users/register
Gateway -> UserService: 转发请求
UserService -> DB: 保存用户信息
DB --> UserService: 返回结果
UserService --> Gateway: 注册成功
Gateway --> FE: 200 OK
FE --> User: 注册成功提示

== 登录 ==
User -> FE: 输入账号密码
FE -> Gateway: POST /api/v1/auth/login
Gateway -> UserService: 验证凭证
UserService -> DB: 查询用户
DB --> UserService: 返回用户数据
UserService -> UserService: 生成 JWT
UserService --> Gateway: 返回 Token
Gateway --> FE: 200 OK + Token
FE -> FE: 存储 Token
FE --> User: 登录成功跳转

@enduml
\`\`\`

## 4. 部署架构

\`\`\`plantuml
@startuml
!theme plain

node "K8s Cluster" {
  node "Node 1" {
    [Pod: User Service] as Pod1
    [Pod: Order Service] as Pod2
  }
  node "Node 2" {
    [Pod: Product Service] as Pod3
    [Pod: Payment Service] as Pod4
  }
  [Ingress Controller] as Ingress
  [Service Registry] as Registry
}

database "MySQL Cluster" as MySQL

database "Redis Cluster" as Redis

Ingress --> Pod1
Ingress --> Pod2
Ingress --> Pod3
Ingress --> Pod4
Pod1 --> Registry
Pod2 --> Registry
Pod3 --> Registry
Pod4 --> Registry
Pod1 --> MySQL
Pod2 --> MySQL
Pod3 --> MySQL
Pod4 --> MySQL
Pod1 --> Redis
Pod2 --> Redis

@enduml
\`\`\`

## 5. 接口设计

### 5.1 用户服务接口

| 接口 | 方法 | 描述 |
|------|------|------|
| /api/v1/users | POST | 创建用户 |
| /api/v1/users/{id} | GET | 获取用户信息 |
| /api/v1/users/{id} | PUT | 更新用户信息 |

## 6. 安全设计

- HTTPS 传输加密
- JWT 身份认证
- API 限流
- 敏感数据加密存储
`,
  },
  {
    id: 'demo-user-story',
    name: '用户故事模板',
    description: '敏捷开发用户故事模板，用于描述功能需求',
    category: '敏捷',
    tags: ['用户故事', '敏捷', '需求'],
    author: '系统',
    content: `# 用户故事 - [故事标题]

## 故事描述

**作为** [角色]
**我希望** [功能]
**以便** [价值]

## 验收标准 (AC)

### AC1: [场景描述]
**Given** [前置条件]
**When** [操作]
**Then** [预期结果]

### AC2: [场景描述]
**Given** [前置条件]
**When** [操作]
**Then** [预期结果]

## 业务流程

\`\`\`plantuml
@startuml
!theme plain

|用户|
start
:执行操作;

|系统|
:处理请求;
:验证数据;

if (验证通过?) then (是)
  :执行业务逻辑;
  :返回成功响应;
else (否)
  :返回错误信息;
endif

|用户|
:查看结果;

stop

@enduml
\`\`\`

## 界面原型

[原型图或线框图]

## 技术要点

- [ ] 技术点1
- [ ] 技术点2

## 依赖关系

- 依赖故事1: #123
- 依赖故事2: #124

## 估算

- 故事点: [3/5/8/13]
- 预计工期: [X 天]
`,
  },
  {
    id: 'demo-api-doc',
    name: 'API 接口文档',
    description: '标准 API 接口文档模板，包含接口定义、参数说明、示例等',
    category: '文档',
    tags: ['API', '接口', '文档'],
    author: '系统',
    content: `# API 接口文档

## 接口基本信息

| 项目 | 内容 |
|------|------|
| 接口名称 | [接口名称] |
| 接口地址 | \`/api/v1/xxx\` |
| 请求方法 | POST |
| 认证方式 | Bearer Token |

## 请求参数

### Header 参数

| 参数名 | 必填 | 类型 | 说明 |
|--------|------|------|------|
| Authorization | 是 | string | Bearer {token} |
| Content-Type | 是 | string | application/json |

### Body 参数

| 参数名 | 必填 | 类型 | 说明 | 示例 |
|--------|------|------|------|------|
| name | 是 | string | 名称 | "test" |
| status | 否 | int | 状态 | 1 |

### 请求示例

\`\`\`json
{
  "name": "示例名称",
  "status": 1,
  "metadata": {
    "key": "value"
  }
}
\`\`\`

## 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| code | int | 状态码 |
| message | string | 提示信息 |
| data | object | 数据对象 |

### 成功响应示例

\`\`\`json
{
  "code": 200,
  "message": "success",
  "data": {
    "id": 1,
    "name": "示例名称",
    "createdAt": "2024-01-01T00:00:00Z"
  }
}
\`\`\`

### 错误响应示例

\`\`\`json
{
  "code": 400,
  "message": "参数错误: name 不能为空",
  "data": null
}
\`\`\`

## 状态码说明

| 状态码 | 说明 |
|--------|------|
| 200 | 成功 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 403 | 无权限 |
| 500 | 服务器内部错误 |

## 时序图

\`\`\`plantuml
@startuml
!theme plain

actor 客户端 as Client
participant "API Gateway" as Gateway
participant "业务服务" as Service
database "数据库" as DB

Client -> Gateway: POST /api/v1/xxx
Gateway -> Gateway: 认证鉴权
Gateway -> Service: 转发请求
Service -> Service: 参数校验
Service -> DB: 查询/写入数据
DB --> Service: 返回结果
Service --> Gateway: 业务响应
Gateway --> Client: JSON 响应

@enduml
\`\`\`
`,
  },
];

// 加载模板
const loadTemplates = async () => {
  isLoading.value = true;
  error.value = null;
  
  try {
    if (props.templateUrl && props.templateUrl.includes('demo')) {
      // 使用演示数据
      await new Promise(resolve => setTimeout(resolve, 500));
      templates.value = demoTemplates;
    } else if (props.templateUrl) {
      // 调用远程接口
      const result = await invoke('fetch_remote_templates', { 
        url: props.templateUrl 
      }) as RemoteTemplate[];
      templates.value = result;
    } else {
      // 默认使用演示数据
      templates.value = demoTemplates;
    }
  } catch (err) {
    console.error('加载模板失败:', err);
    error.value = '加载模板失败: ' + err;
    // 失败时回退到演示数据
    templates.value = demoTemplates;
  } finally {
    isLoading.value = false;
  }
};

// 过滤后的模板
const filteredTemplates = ref<RemoteTemplate[]>([]);

watch([templates, searchQuery, selectedCategory], () => {
  let result = templates.value;
  
  // 搜索过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(t => 
      t.name.toLowerCase().includes(query) ||
      t.description.toLowerCase().includes(query) ||
      t.tags?.some(tag => tag.toLowerCase().includes(query))
    );
  }
  
  // 分类过滤
  if (selectedCategory.value !== 'all') {
    result = result.filter(t => t.category === selectedCategory.value);
  }
  
  filteredTemplates.value = result;
}, { immediate: true });

// 分类列表
const categories = computed(() => {
  const cats = new Set(templates.value.map(t => t.category).filter(Boolean));
  return ['all', ...Array.from(cats)];
});

// 选择模板
const selectTemplate = (template: RemoteTemplate) => {
  selectedTemplate.value = template;
  previewMode.value = true;
};

// 确认使用模板
const confirmUseTemplate = () => {
  if (selectedTemplate.value) {
    emit('select', selectedTemplate.value);
    closeModal();
  }
};

// 关闭弹窗
const closeModal = () => {
  previewMode.value = false;
  selectedTemplate.value = null;
  emit('close');
};

// 获取分类名称
const getCategoryName = (cat: string) => {
  const names: Record<string, string> = {
    'all': '全部',
    '规范': '规范',
    '文档': '文档',
    '架构': '架构',
    '敏捷': '敏捷',
  };
  return names[cat] || cat;
};

onMounted(() => {
  if (props.show) {
    loadTemplates();
  }
});

watch(() => props.show, (newVal) => {
  if (newVal) {
    loadTemplates();
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- 背景遮罩 -->
        <div 
          class="absolute inset-0 bg-black/50 backdrop-blur-sm" 
          @click="closeModal"
        />
        
        <!-- 弹窗内容 -->
        <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <!-- 头部 -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-gradient-to-r from-blue-50 to-indigo-50">
            <div class="flex items-center gap-3">
              <div class="p-2 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl">
                <FolderOpen :size="20" class="text-white" />
              </div>
              <div>
                <h2 class="text-lg font-bold text-gray-800">模板库</h2>
                <p class="text-xs text-gray-500">选择模板快速开始</p>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="loadTemplates"
                :disabled="isLoading"
                class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
                title="刷新"
              >
                <RefreshCw :size="18" :class="isLoading ? 'animate-spin' : ''" class="text-gray-500" />
              </button>
              <button 
                @click="closeModal"
                class="p-2 hover:bg-gray-200/50 rounded-lg transition-colors"
              >
                <X :size="20" class="text-gray-500" />
              </button>
            </div>
          </div>
          
          <!-- 搜索和过滤 -->
          <div class="p-4 border-b border-gray-200 bg-gray-50/50">
            <div class="flex gap-3">
              <div class="flex-1 relative">
                <Search :size="18" class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="搜索模板..."
                  class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-xl focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                />
              </div>
              <select
                v-model="selectedCategory"
                class="px-4 py-2 border border-gray-300 rounded-xl focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all bg-white"
              >
                <option v-for="cat in categories" :key="cat" :value="cat">
                  {{ getCategoryName(cat) }}
                </option>
              </select>
            </div>
          </div>
          
          <!-- 内容区域 -->
          <div class="flex-1 overflow-hidden flex">
            <!-- 模板列表 -->
            <div class="flex-1 overflow-y-auto p-4">
              <!-- 加载中 -->
              <div v-if="isLoading" class="flex flex-col items-center justify-center h-64">
                <RefreshCw :size="32" class="animate-spin text-blue-500 mb-4" />
                <p class="text-gray-500">加载模板中...</p>
              </div>
              
              <!-- 错误 -->
              <div v-else-if="error" class="flex flex-col items-center justify-center h-64 text-center">
                <AlertCircle :size="32" class="text-red-500 mb-4" />
                <p class="text-red-600 mb-2">{{ error }}</p>
                <button
                  @click="loadTemplates"
                  class="text-blue-600 hover:underline"
                >
                  重试
                </button>
              </div>
              
              <!-- 空状态 -->
              <div v-else-if="filteredTemplates.length === 0" class="flex flex-col items-center justify-center h-64 text-center">
                <FileText :size="32" class="text-gray-300 mb-4" />
                <p class="text-gray-500">没有找到匹配的模板</p>
              </div>
              
              <!-- 模板网格 -->
              <div v-else class="grid grid-cols-2 gap-3">
                <div
                  v-for="template in filteredTemplates"
                  :key="template.id"
                  @click="selectTemplate(template)"
                  :class="[
                    'p-4 rounded-xl border-2 cursor-pointer transition-all hover:shadow-md',
                    selectedTemplate?.id === template.id
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-blue-300'
                  ]"
                >
                  <div class="flex items-start justify-between mb-2">
                    <h3 class="font-semibold text-gray-800">{{ template.name }}</h3>
                    <span v-if="template.category" class="text-xs px-2 py-1 bg-gray-100 text-gray-600 rounded-full">
                      {{ template.category }}
                    </span>
                  </div>
                  <p class="text-sm text-gray-500 line-clamp-2 mb-3">{{ template.description }}</p>
                  <div class="flex items-center justify-between">
                    <div class="flex gap-1 flex-wrap">
                      <span
                        v-for="tag in template.tags?.slice(0, 3)"
                        :key="tag"
                        class="text-xs px-1.5 py-0.5 bg-blue-50 text-blue-600 rounded"
                      >
                        {{ tag }}
                      </span>
                    </div>
                    <span v-if="template.author" class="text-xs text-gray-400">
                      by {{ template.author }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- 预览面板 -->
            <div 
              v-if="previewMode && selectedTemplate"
              class="w-96 border-l border-gray-200 bg-gray-50 flex flex-col"
            >
              <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-white">
                <h3 class="font-semibold text-gray-800">预览</h3>
                <button @click="previewMode = false" class="text-gray-400 hover:text-gray-600">
                  <X :size="18" />
                </button>
              </div>
              <div class="flex-1 overflow-y-auto p-4">
                <h2 class="text-lg font-bold text-gray-800 mb-2">{{ selectedTemplate.name }}</h2>
                <p class="text-sm text-gray-500 mb-4">{{ selectedTemplate.description }}</p>
                
                <div class="bg-white rounded-xl border border-gray-200 p-4 mb-4">
                  <pre class="text-xs text-gray-600 overflow-auto max-h-64 whitespace-pre-wrap">{{ selectedTemplate.content.substring(0, 500) }}...</pre>
                </div>
                
                <button
                  @click="confirmUseTemplate"
                  class="w-full py-2.5 bg-gradient-to-r from-blue-500 to-indigo-600 text-white rounded-xl font-medium hover:opacity-90 transition-all flex items-center justify-center gap-2"
                >
                  <Download :size="18" />
                  使用此模板
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script lang="ts">
import { computed } from 'vue';
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .relative,
.modal-leave-to .relative {
  transform: scale(0.95);
  opacity: 0;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
