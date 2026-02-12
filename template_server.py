#!/usr/bin/env python3
"""
FastAPI 模板服务模拟器
用于测试 Tauri 编辑器中的 HTTP 调用功能

启动命令: python template_server.py
默认端口: 8000
"""

import json
import random
import time
from datetime import datetime
from typing import List, Optional

from fastapi import FastAPI, HTTPException, Request, Response, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field

# ==================== 数据模型 ====================


class Template(BaseModel):
    """模板数据结构"""

    id: str = Field(..., description="模板ID")
    name: str = Field(..., description="模板名称")
    description: str = Field(..., description="模板描述")
    content: str = Field(..., description="模板内容")
    category: Optional[str] = Field("general", description="模板分类")
    created_at: Optional[str] = Field(None, description="创建时间")
    updated_at: Optional[str] = Field(None, description="更新时间")


class TemplateUpload(BaseModel):
    """模板上传请求"""

    id: str = Field(..., description="模板ID")
    name: str = Field(..., description="模板名称")
    description: str = Field(..., description="模板描述")
    content: str = Field(..., description="模板内容")


class ApiResponse(BaseModel):
    """API 响应格式"""

    success: bool = Field(..., description="请求是否成功")
    message: str = Field(..., description="响应消息")
    data: Optional[dict] = Field(None, description="响应数据")
    timestamp: str = Field(..., description="时间戳")


# ==================== 模拟数据 ====================


def generate_mock_templates() -> List[Template]:
    """生成模拟模板数据"""
    templates = [
        Template(
            id="api-design-template",
            name="API 设计规范",
            description="RESTful API 设计规范模板，包含命名规范、状态码、版本控制等",
            content="# API 设计规范\n\n## 设计原则\n1. 使用名词复数形式表示资源\n2. 使用 HTTP 方法表示操作类型\n3. 使用状态码表示请求结果\n\n## 命名规范\n- 资源命名：小写下划线，复数形式\n- 版本控制：/api/v1/resource\n- 查询参数：page, limit, sort, filter\n\n## 状态码\n- 200: 成功\n- 201: 创建成功\n- 400: 请求错误\n- 401: 未授权\n- 403: 禁止访问\n- 404: 资源不存在\n- 500: 服务器错误",
            category="backend",
            created_at="2024-01-15T10:30:00Z",
            updated_at="2024-01-20T14:25:00Z",
        ),
        Template(
            id="database-design-template",
            name="数据库设计规范",
            description="数据库表结构设计模板，包含命名规范、索引、约束等",
            content="# 数据库设计规范\n\n## 命名规范\n- 表名：小写下划线，复数形式\n- 字段名：小写下划线\n- 索引名：idx_表名_字段名\n- 外键名：fk_表名_字段名\n\n## 数据类型\n- 主键：BIGINT UNSIGNED AUTO_INCREMENT\n- 字符串：VARCHAR(255)\n- 文本：TEXT\n- 时间戳：TIMESTAMP\n- 布尔值：TINYINT(1)\n\n## 索引策略\n- 主键必须有索引\n- 外键必须有索引\n- 查询频繁的字段加索引\n- 联合索引注意顺序",
            category="database",
            created_at="2024-01-10T09:15:00Z",
            updated_at="2024-01-18T11:40:00Z",
        ),
        Template(
            id="prd-template",
            name="PRD 文档模板",
            description="产品需求文档标准模板，包含背景、需求、验收标准等",
            content="# PRD 文档\n\n## 1. 背景\n描述项目背景和目标\n\n## 2. 需求描述\n详细描述功能需求\n\n## 3. 用户故事\n- 作为 [角色]，我想要 [功能]，以便于 [价值]\n\n## 4. 验收标准\n- [ ] 标准1\n- [ ] 标准2\n- [ ] 标准3\n\n## 5. 非功能性需求\n- 性能要求\n- 安全性要求\n- 兼容性要求\n\n## 6. 时间计划\n- 开始时间：YYYY-MM-DD\n- 结束时间：YYYY-MM-DD",
            category="documentation",
            created_at="2024-01-05T14:20:00Z",
            updated_at="2024-01-12T16:35:00Z",
        ),
        Template(
            id="architecture-design-template",
            name="系统架构设计",
            description="系统架构设计文档模板，包含架构图、组件说明、部署方案等",
            content='# 系统架构设计\n\n## 架构图\n```plantuml\n@startuml\n!theme plain\n\ncomponent "前端" as FE\ncomponent "API网关" as Gateway\ncomponent "用户服务" as UserService\ncomponent "订单服务" as OrderService\ndatabase "MySQL" as DB\n\nFE -> Gateway: HTTP请求\nGateway -> UserService: 用户认证\nGateway -> OrderService: 订单处理\nUserService -> DB: 数据存储\nOrderService -> DB: 数据存储\n@enduml\n```\n\n## 组件说明\n1. 前端：Vue 3 + TypeScript\n2. API网关：Nginx + OpenResty\n3. 用户服务：Spring Boot\n4. 订单服务：Spring Boot\n5. 数据库：MySQL 8.0\n\n## 部署方案\n- 开发环境：Docker Compose\n- 测试环境：Kubernetes\n- 生产环境：Kubernetes + Helm',
            category="architecture",
            created_at="2024-01-08T11:45:00Z",
            updated_at="2024-01-22T09:10:00Z",
        ),
        Template(
            id="test-plan-template",
            name="测试计划模板",
            description="软件测试计划模板，包含测试范围、策略、资源、进度等",
            content="# 测试计划\n\n## 测试范围\n- 功能测试\n- 性能测试\n- 安全测试\n- 兼容性测试\n\n## 测试策略\n### 单元测试\n- 覆盖率要求：≥80%\n- 框架：JUnit / pytest\n\n### 集成测试\n- API 接口测试\n- 数据库集成测试\n\n### 系统测试\n- 端到端测试\n- 用户场景测试\n\n## 测试资源\n- 测试环境：test.example.com\n- 测试数据：mock_data.sql\n- 测试工具：Postman, JMeter\n\n## 进度计划\n- 单元测试：第1-2周\n- 集成测试：第3周\n- 系统测试：第4周",
            category="testing",
            created_at="2024-01-12T13:30:00Z",
            updated_at="2024-01-25T15:45:00Z",
        ),
        Template(
            id="deployment-guide-template",
            name="部署指南模板",
            description="系统部署指南模板，包含环境准备、配置、部署步骤等",
            content='# 部署指南\n\n## 环境要求\n- 操作系统：Ubuntu 20.04+\n- 内存：≥8GB\n- 存储：≥50GB\n- 网络：公网IP\n\n## 依赖安装\n```bash\n# 安装 Docker\ncurl -fsSL https://get.docker.com -o get-docker.sh\nsudo sh get-docker.sh\n\n# 安装 Docker Compose\nsudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose\nsudo chmod +x /usr/local/bin/docker-compose\n```\n\n## 配置说明\n### 数据库配置\n```yaml\ndatabase:\n  host: localhost\n  port: 3306\n  username: admin\n  password: secret\n  database: app_db\n```\n\n## 部署步骤\n1. 克隆代码仓库\n2. 修改配置文件\n3. 构建 Docker 镜像\n4. 启动容器\n5. 验证部署',
            category="deployment",
            created_at="2024-01-18T16:20:00Z",
            updated_at="2024-01-28T10:15:00Z",
        ),
    ]
    return templates


# ==================== FastAPI 应用 ====================

app = FastAPI(
    title="模板服务模拟器",
    description="用于测试 Tauri 编辑器 HTTP 调用的模拟服务",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc",
)

# 启用 CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # 允许所有来源，生产环境应限制
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 全局状态
mock_templates = generate_mock_templates()


# ==================== 中间件 ====================


@app.middleware("http")
async def add_process_time_header(request: Request, call_next):
    """添加请求处理时间头"""
    start_time = time.time()
    response = await call_next(request)
    process_time = time.time() - start_time

    # 模拟网络延迟（50-500ms）
    delay = random.uniform(0.05, 0.5)
    time.sleep(delay)

    response.headers["X-Process-Time"] = str(process_time + delay)
    response.headers["X-Server-Name"] = "Template Mock Server"
    return response


# ==================== 健康检查 ====================


@app.get("/", tags=["健康检查"])
async def root():
    """根路径，返回服务信息"""
    return {
        "service": "Template Mock Server",
        "version": "1.0.0",
        "status": "running",
        "timestamp": datetime.now().isoformat(),
        "endpoints": {
            "templates": "/api/templates",
            "template_by_id": "/api/templates/{id}",
            "upload": "/api/templates/upload",
            "search": "/api/templates/search?q={query}",
            "health": "/health",
        },
    }


@app.get("/health", tags=["健康检查"])
async def health_check():
    """健康检查端点"""
    return JSONResponse(
        status_code=status.HTTP_200_OK,
        content={
            "status": "healthy",
            "timestamp": datetime.now().isoformat(),
            "template_count": len(mock_templates),
        },
    )


# ==================== 模板 API ====================


@app.get("/api/templates", response_model=List[Template], tags=["模板管理"])
async def get_templates(
    skip: int = 0, limit: int = 100, category: Optional[str] = None
):
    """
    获取模板列表

    - **skip**: 跳过多少条记录（分页）
    - **limit**: 返回多少条记录（分页）
    - **category**: 按分类过滤
    """
    # 按分类过滤
    filtered_templates = mock_templates
    if category:
        filtered_templates = [t for t in mock_templates if t.category == category]

    # 分页
    result = filtered_templates[skip : skip + limit]

    # 随机模拟一些错误（10%概率）
    if random.random() < 0.1:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="模拟服务器错误：数据库连接失败",
        )

    return result


@app.get("/api/templates/{template_id}", response_model=Template, tags=["模板管理"])
async def get_template_by_id(template_id: str):
    """
    根据ID获取模板详情
    """
    for template in mock_templates:
        if template.id == template_id:
            return template

    raise HTTPException(
        status_code=status.HTTP_404_NOT_FOUND, detail=f"模板 '{template_id}' 不存在"
    )


@app.post("/api/templates/search", response_model=List[Template], tags=["模板管理"])
async def search_templates(q: str, skip: int = 0, limit: int = 50):
    """
    搜索模板

    - **q**: 搜索关键词
    - **skip**: 跳过多少条记录
    - **limit**: 返回多少条记录
    """
    if not q or len(q.strip()) < 2:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST, detail="搜索关键词至少需要2个字符"
        )

    q_lower = q.lower()
    results = []

    for template in mock_templates:
        if (
            q_lower in template.name.lower()
            or q_lower in template.description.lower()
            or q_lower in template.category.lower()
        ):
            results.append(template)

    # 分页
    return results[skip : skip + limit]


@app.post("/api/templates/upload", response_model=ApiResponse, tags=["模板管理"])
async def upload_template(template: TemplateUpload, request: Request):
    """
    上传新模板

    - **template**: 模板数据
    """
    # 检查模板是否已存在
    for existing in mock_templates:
        if existing.id == template.id:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail=f"模板ID '{template.id}' 已存在",
            )

    # 创建新模板
    new_template = Template(
        id=template.id,
        name=template.name,
        description=template.description,
        content=template.content,
        category="user_uploaded",
        created_at=datetime.now().isoformat(),
        updated_at=datetime.now().isoformat(),
    )

    mock_templates.append(new_template)

    return ApiResponse(
        success=True,
        message=f"模板 '{template.name}' 上传成功",
        data={"template_id": template.id},
        timestamp=datetime.now().isoformat(),
    )


@app.delete(
    "/api/templates/{template_id}", response_model=ApiResponse, tags=["模板管理"]
)
async def delete_template(template_id: str):
    """
    删除模板

    - **template_id**: 模板ID
    """
    global mock_templates

    original_count = len(mock_templates)
    mock_templates = [t for t in mock_templates if t.id != template_id]

    if len(mock_templates) == original_count:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail=f"模板 '{template_id}' 不存在"
        )

    return ApiResponse(
        success=True,
        message=f"模板 '{template_id}' 删除成功",
        timestamp=datetime.now().isoformat(),
    )


# ==================== 测试端点 ====================


@app.get("/api/test/success", tags=["测试"])
async def test_success():
    """测试成功响应"""
    return {
        "status": "success",
        "message": "请求成功",
        "data": {"test": "value"},
        "timestamp": datetime.now().isoformat(),
    }


@app.get("/api/test/error/{status_code}", tags=["测试"])
async def test_error(status_code: int = 500):
    """测试错误响应"""
    error_messages = {
        400: "错误的请求",
        401: "未授权",
        403: "禁止访问",
        404: "资源不存在",
        500: "服务器内部错误",
        502: "网关错误",
        503: "服务不可用",
    }

    message = error_messages.get(status_code, f"HTTP {status_code} 错误")

    raise HTTPException(status_code=status_code, detail=message)


@app.get("/api/test/delay/{seconds}", tags=["测试"])
async def test_delay(seconds: float = 1.0):
    """测试延迟响应"""
    if seconds > 10:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST, detail="延迟时间不能超过10秒"
        )

    time.sleep(seconds)

    return {
        "status": "success",
        "message": f"延迟 {seconds} 秒后响应",
        "delay_seconds": seconds,
        "timestamp": datetime.now().isoformat(),
    }


# ==================== 启动脚本 ====================

if __name__ == "__main__":
    import uvicorn

    print("=" * 60)
    print("模板服务模拟器")
    print("=" * 60)
    print(f"服务地址: http://localhost:8000")
    print(f"API文档: http://localhost:8000/docs")
    print(f"模板列表: http://localhost:8000/api/templates")
    print(f"健康检查: http://localhost:8000/health")
    print("=" * 60)
    print("可用端点:")
    print("  GET  /api/templates           - 获取模板列表")
    print("  GET  /api/templates/{id}      - 获取模板详情")
    print("  POST /api/templates/search    - 搜索模板")
    print("  POST /api/templates/upload    - 上传模板")
    print("  DELETE /api/templates/{id}    - 删除模板")
    print("=" * 60)

    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")
