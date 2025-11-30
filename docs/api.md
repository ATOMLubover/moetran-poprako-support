# Moetran Support API 文档

所有返回统一包装：

```json
// 成功示例
{
    "code": 200,
    "data": { /* 与具体接口相关的结构 */ }
}

// 失败示例
{
    "code": 401,
    "message": "Unauthorized"
}
```

HTTP 状态码与 `code` 字段保持一致。

**认证**：除 `/api/v1/sync` 外，其余接口都需要在 Header 中携带：

```text
Authorization: Bearer <JWT>
```

JWT `sub` 字段即用户 `user_id`。

---

## User 部分

### 1. 同步用户信息

用于将外部用户数据同步到本服务数据库。若用户已存在则更新；若不存在则创建新用户并返回 JWT token。

| 方法 | 路径                | 认证   |
| ---- | ------------------- | ------ |
| POST | `/api/v1/sync`      | 不需要 |

请求体（JSON）：

```json
{
    "user_id": "user_123",
    "username": "alice",
    "email": "alice@example.com",
    "password": "secret"
}
```

成功响应（200 或 201）：

```json
{
    "code": 200,
    "data": {
        "token": "<jwt-token>"
    }
}
```

错误响应：

| 场景             | code |
| ---------------- | ---- |
| JSON 解析错误    | 422  |
| 内部错误         | 500  |

示例 cURL：

```bash
curl -X POST https://api.example.com/api/v1/sync \
    -H "Content-Type: application/json" \
    -d '{
      "user_id":"user_123",
      "username":"alice",
      "email":"alice@example.com",
      "password":"secret"
    }'
```

### 2. 获取用户信息

返回当前登录用户的基本资料及其所属团队列表。

| 方法 | 路径                    | 认证 |
| ---- | ----------------------- | ---- |
| GET  | `/api/v1/users/{user_id}` | 需要 |

成功响应：

```json
{
    "code": 200,
    "data": {
        "user_id": "user_123",
        "username": "alice",
        "email": "alice@example.com",
        "teams": [
            {
                "team_id": "team_a",
                "team_name": "汉化组A"
            }
        ]
    }
}
```

错误响应：

| 场景         | code |
| ------------ | ---- |
| 未认证       | 401  |
| 用户不存在   | 404  |
| 内部错误     | 500  |

示例 cURL：

```bash
curl -H "Authorization: Bearer <jwt>" \
    https://api.example.com/api/v1/users/user_123
```

---

## Member 部分

### 3. 获取自己在特定团队中的成员信息

根据当前登录用户与指定 `team_id`，返回该用户在该团队的角色标记和成员 ID。

| 方法 | 路径                     | 认证 | 查询参数           |
| ---- | ------------------------ | ---- | ------------------ |
| GET  | `/api/v1/members/info`   | 需要 | `team_id=<string>` |

成功响应：

```json
{
    "code": 200,
    "data": {
        "member_id": "member_789",
        "user_id": "user_123",
        "username": "alice",
        "is_admin": false,
        "is_translator": true,
        "is_proofreader": false,
        "is_typesetter": false,
        "is_principal": false
    }
}
```

错误响应：

| 场景                       | code |
| -------------------------- | ---- |
| 未认证                     | 401  |
| team_id 缺失               | 400  |
| 成员不存在                 | 404  |
| 内部错误                   | 500  |

示例 cURL：

```bash
curl -G https://api.example.com/api/v1/members/info \
    -H "Authorization: Bearer <jwt>" \
    --data-urlencode "team_id=team_a"
```

### 4. 搜索/筛选成员列表

按团队、职位及用户名模糊查询成员列表。支持两种调用方式。

**1) JSON POST（推荐）**

| 方法 | 路径                        | 认证 |
| ---- | --------------------------- | ---- |
| POST | `/api/v1/members/search`    | 需要 |

请求体（JSON）：

```json
{
    "team_id": "team_a",
    "position": "translator",
    "fuzzy_name": "ali",
    "page": 1,
    "limit": 20
}
```

字段说明（接受 camelCase 别名）：

- `team_id`（必填）：所属团队 ID
- `position`（可选）：职位过滤，取值：`translator` / `proofreader` / `typesetter` / `principal`
- `fuzzy_name`（可选）：用户名模糊匹配关键字
- `page`：页码，从 1 开始，默认 1
- `limit`：每页条数，默认 10

成功响应：

```json
{
    "code": 200,
    "data": [
        {
            "member_id": "member_1",
            "user_id": "user_123",
            "username": "alice"
        },
        {
            "member_id": "member_2",
            "user_id": "user_456",
            "username": "alice_2"
        }
    ]
}
```

错误响应：

| 场景              | code |
| ----------------- | ---- |
| 未认证            | 401  |
| team_id 缺失      | 422  |
| position 非法取值 | 400  |
| 内部错误          | 500  |

示例 cURL（POST）：

```bash
curl -X POST https://api.example.com/api/v1/members/search \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "team_id":"team_a",
      "position":"translator",
      "fuzzy_name":"ali",
      "page":1,
      "limit":20
    }'
```

**2) Query GET（兼容旧接口）**

| 方法 | 路径                 | 认证 | 查询参数                                    |
| ---- | -------------------- | ---- | ------------------------------------------- |
| GET  | `/api/v1/members`    | 需要 | `team_id`, `position`, `fuzzy_name`, `page`, `limit` |

行为与 POST `/api/v1/members/search` 相同，仅以 query 参数形式传递。

示例 cURL（GET）：

```bash
curl -G https://api.example.com/api/v1/members \
    -H "Authorization: Bearer <jwt>" \
    --data-urlencode "team_id=team_a" \
    --data-urlencode "position=translator" \
    --data-urlencode "fuzzy_name=ali" \
    --data-urlencode "page=1" \
    --data-urlencode "limit=20"
```

---

## ProjSet 部分

### 5. 创建项目集

在指定团队下创建一个新的项目集。仅团队管理员可创建。首先在 Moetran 上创建对应 project-set，再在本服务中记录 serial。

| 方法 | 路径                     | 认证 |
| ---- | ------------------------ | ---- |
| POST | `/api/v1/projsets`       | 需要 |

请求体（JSON）：

```json
{
    "projset_name": "主线文本",
    "projset_description": "主线剧情相关文本",
    "team_id": "team_a",
    "mtr_token": "<moetran-access-token>"
}
```

字段说明（接受 camelCase 别名）：

- `projset_name`（必填）：项目集名称
- `projset_description`（可选）：项目集描述
- `team_id`（必填）：所属团队 ID
- `mtr_token`（必填）：Moetran 访问 token，用于调用外部 API

成功响应（201）：

```json
{
    "code": 201,
    "data": {
        "projset_serial": 3
    }
}
```

错误响应：

| 场景                   | code |
| ---------------------- | ---- |
| 未认证                 | 401  |
| 非管理员用户           | 403  |
| JSON 解析错误          | 422  |
| Moetran 接口失败       | 500  |
| 内部错误               | 500  |

示例 cURL：

```bash
curl -X POST https://api.example.com/api/v1/projsets \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "projset_name":"主线文本",
      "projset_description":"主线剧情相关文本",
      "team_id":"team_a",
      "mtr_token":"<moetran-token>"
    }'
```

### 6. 列出团队下的项目集

列出指定团队下的所有项目集。

| 方法 | 路径                 | 认证 | 查询参数           |
| ---- | -------------------- | ---- | ------------------ |
| GET  | `/api/v1/projsets`   | 需要 | `team_id=<string>` |

成功响应：

```json
{
    "code": 200,
    "data": {
        "projsets": [
            {
                "projset_id": "projset_123",
                "projset_name": "主线文本",
                "projset_description": "主线剧情相关文本",
                "projset_serial": 3,
                "team_id": "team_a"
            }
        ]
    }
}
```

错误响应：

| 场景           | code |
| -------------- | ---- |
| 未认证         | 401  |
| team_id 缺失   | 400  |
| 团队不存在     | 404  |
| 内部错误       | 500  |

示例 cURL：

```bash
curl -G https://api.example.com/api/v1/projsets \
    -H "Authorization: Bearer <jwt>" \
    --data-urlencode "team_id=team_a"
```

---

## Proj 部分

### 7. 创建项目

在已有项目集中创建一个具体项目。仅团队管理员可创建。会先调用 Moetran 创建实际项目，再记录到本系统。项目创建者自动成为项目负责人（principal）。

| 方法 | 路径             | 认证 |
| ---- | ---------------- | ---- |
| POST | `/api/v1/projs`  | 需要 |

请求体（JSON）：

```json
{
    "proj_name": "章节1对话",
    "proj_description": "第一章主要对话文本",
    "team_id": "team_a",
    "projset_id": "projset_123",
    "mtr_auth": "<moetran-access-token>",
    "source_language": "ja",
    "target_languages": ["zh_CN"],
    "default_role": "63d87c24b8bebd75ff934267"
}
```

字段说明（接受 camelCase 别名）：

- `proj_name`（必填）：项目名称
- `proj_description`（可选）：项目描述
- `team_id`（必填）：所属团队 ID
- `projset_id`（必填）：所属项目集 ID
- `mtr_auth`（必填）：Moetran 访问 token
- `source_language`（必填）：源语言代码（如 "ja"）
- `target_languages`（必填）：目标语言代码数组（如 ["zh_CN"]）
- `default_role`（必填）：新申请默认角色 ID（见 MtrRole 枚举）

成功响应（201）：

```json
{
    "code": 201,
    "data": {
        "proj_id": "proj_id_1",
        "proj_serial": 12,
        "projset_index": 5
    }
}
```

错误响应：

| 场景                | code |
| ------------------- | ---- |
| 未认证              | 401  |
| 非管理员用户        | 403  |
| 项目集不存在        | 404  |
| JSON 解析错误       | 422  |
| Moetran 接口失败    | 500  |
| 内部错误            | 500  |

示例 cURL：

```bash
curl -X POST https://api.example.com/api/v1/projs \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "proj_name":"章节1对话",
      "proj_description":"第一章主要对话文本",
      "team_id":"team_a",
      "projset_id":"projset_123",
      "mtr_auth":"<moetran-token>",
      "source_language":"ja",
      "target_languages":["zh_CN"],
      "default_role":"63d87c24b8bebd75ff934267"
    }'
```

### 8. 搜索/筛选项目列表

通用的项目搜索接口，支持按项目 ID 列表直接查询，或按多条件筛选并分页返回项目信息与参与成员列表。

| 方法 | 路径             | 认证 |
| ---- | ---------------- | ---- |
| POST | `/api/v1/projs/search` | 需要 |

请求体（JSON）：

```json
{
    "proj_ids": ["proj_id_1", "proj_id_2"],
    "fuzzy_proj_name": "章节1",
    "projset_ids": ["projset_123"],
    "translating_status": 0,
    "proofreading_status": 0,
    "typesetting_status": 0,
    "reviewing_status": 0,
    "is_published": false,
    "member_ids": ["member_1", "member_2"],
    "time_start": 1690000000,
    "page": 1,
    "limit": 10
}
```

字段说明（接受 camelCase 别名）：

- `proj_ids`（可选）：若提供且非空，则直接按 ID 列表查询（忽略其他条件）
- `fuzzy_proj_name`（可选）：项目名模糊匹配（ILIKE）
- `projset_ids`（可选）：按项目集 ID 列表过滤
- `translating_status`（可选）：翻译流程状态（0/1/2）
- `proofreading_status`（可选）：校对流程状态（0/1/2）
- `typesetting_status`（可选）：排版流程状态（0/1/2）
- `reviewing_status`（可选）：评审流程状态（0/1/2）
- `is_published`（可选）：是否已发布
- `member_ids`（可选）：仅返回包含指定成员的项目
- `time_start`（可选）：Unix 时间戳（秒），返回创建时间 >= 该时间的项目
- `page`：页码，从 1 开始，默认 1
- `limit`：每页条数，默认 10

成功响应：

```json
{
    "code": 200,
    "data": [
        {
            "proj_id": "proj_id_1",
            "proj_name": "【3-5】章节1对话",
            "description": "第一章主要对话文本",
            "projset_id": "projset_123",
            "projset_serial": 3,
            "projset_index": 5,
            "translating_status": 0,
            "proofreading_status": 0,
            "typesetting_status": 0,
            "reviewing_status": 0,
            "is_published": false,
            "members": [
                {
                    "member_id": "member_1",
                    "username": "alice",
                    "is_admin": false,
                    "is_translator": true,
                    "is_proofreader": false,
                    "is_typesetter": false,
                    "is_principal": true
                }
            ]
        }
    ]
}
```

错误响应：

| 场景            | code |
| --------------- | ---- |
| 未认证          | 401  |
| JSON 解析错误   | 422  |
| 内部错误        | 500  |

示例 cURL：

```bash
curl -X POST https://api.example.com/api/v1/projs/search \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "fuzzy_proj_name":"章节1",
      "projset_ids":["projset_123"],
      "page":1,
      "limit":10
    }'
```

### 9. 更新项目流程状态

仅项目负责人（principal）可以修改项目的流程状态。状态值采用枚举：`0=未开始`、`1=进行中`、`2=已完成`。

| 方法 | 路径                             | 认证 |
| ---- | -------------------------------- | ---- |
| PUT  | `/api/v1/projs/{proj_id}/status` | 需要 |

请求体（JSON）：

```json
{
    "proj_id": "proj_id_1",
    "status_type": "translating",
    "new_status": 1
}
```

字段说明（接受 camelCase 别名）：

- `proj_id`：项目 ID（会被路径中的值覆盖）
- `status_type`（必填）：流程类型，取值：`translating` / `proofreading` / `typesetting` / `reviewing`
- `new_status`（必填）：新状态（0/1/2）

成功响应（204）：

```json
{
    "code": 204,
    "data": null
}
```

错误响应：

| 场景         | code |
| ------------ | ---- |
| 未认证       | 401  |
| 非项目负责人 | 403  |
| status_type 非法 | 400  |
| JSON 解析错误 | 422  |
| 内部错误     | 500  |

示例 cURL：

```bash
curl -X PUT https://api.example.com/api/v1/projs/proj_id_1/status \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "status_type":"translating",
      "new_status":1
    }'
```

### 10. 标记项目为已发布

仅项目负责人可以将项目标记为已发布。

| 方法 | 路径                              | 认证 |
| ---- | --------------------------------- | ---- |
| PUT  | `/api/v1/projs/{proj_id}/publish` | 需要 |

请求体：无

成功响应（204）：

```json
{
    "code": 204,
    "data": null
}
```

错误响应：

| 场景         | code |
| ------------ | ---- |
| 未认证       | 401  |
| 非项目负责人 | 403  |
| 项目不存在   | 404  |
| 内部错误     | 500  |

示例 cURL：

```bash
curl -X PUT https://api.example.com/api/v1/projs/proj_id_1/publish \
    -H "Authorization: Bearer <jwt>"
```

---

## Assign 部分

### 11. 指派成员到项目

仅项目负责人（principal）可以为项目指派或更新成员角色。若成员在目标团队中不存在、或不具备请求中的角色能力，则返回错误。

此操作会：
1. 检查本地 `t_proj_assgin` 表中是否已存在该成员分配记录
2. 若不存在，调用 Moetran 邀请/指派接口
3. 最后在本地 `t_proj_assgin` 表中 upsert 分配记录

| 方法 | 路径                             | 认证 |
| ---- | -------------------------------- | ---- |
| POST | `/api/v1/projs/{proj_id}/assign` | 需要 |

请求体（JSON）：

```json
{
    "proj_id": "proj_id_1",
    "member_id": "member_1",
    "mtr_auth": "<moetran-access-token>",
    "is_translator": true,
    "is_proofreader": false,
    "is_typesetter": false
}
```

字段说明（接受 camelCase 别名）：

- `proj_id`：项目 ID（会被路径中的值覆盖）
- `member_id`（必填）：目标成员 ID
- `mtr_auth`（必填）：Moetran 访问 token
- `is_translator`（可选，默认 false）：是否分配翻译角色
- `is_proofreader`（可选，默认 false）：是否分配校对角色
- `is_typesetter`（可选，默认 false）：是否分配排版角色

成功响应（204）：

```json
{
    "code": 204,
    "data": null
}
```

错误响应：

| 场景                               | code |
| ---------------------------------- | ---- |
| 未认证                             | 401  |
| 操作人不是项目负责人               | 403  |
| 成员不属于该项目所在团队           | 404  |
| 成员不具备请求的角色能力           | 400  |
| JSON 解析错误                      | 422  |
| Moetran 邀请接口失败               | 500  |
| 内部错误                           | 500  |

说明：
- 若成员已在本地分配表中存在，则跳过 Moetran 邀请调用，直接更新本地分配记录（幂等）。
- 若 Moetran 返回 `UserAlreadyJoinedError` (code 5003)，服务将跳过该错误并继续进行本地 upsert。

示例 cURL：

```bash
curl -X POST https://api.example.com/api/v1/projs/proj_id_1/assign \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "member_id":"member_1",
      "mtr_auth":"<moetran-token>",
      "is_translator":true,
      "is_proofreader":false,
      "is_typesetter":false
    }'
```

---

## 公共错误语义

| code | 含义                               |
| ---- | ---------------------------------- |
| 200  | 成功                               |
| 201  | 已创建                             |
| 204  | 无内容（操作成功但无返回数据）     |
| 400  | 请求格式错误或业务逻辑错误         |
| 401  | 未认证或凭证无效                   |
| 403  | 权限不足                           |
| 404  | 资源不存在                         |
| 422  | 语义/字段解析失败                  |
| 500  | 服务器内部错误                     |

统一约定：

- 所有失败响应都包含 `message` 字段（说明错误原因）
- 成功响应通常无 `message` 字段
- 所有请求体支持 snake_case 与 camelCase 混用（通过 serde aliases）

---

## 数据类型与枚举

### ProjStatus（项目流程状态）

用于各流程状态字段的整数枚举：

- `0`：`NotStarted`（未开始）
- `1`：`InProgress`（进行中）
- `2`：`Completed`（已完成）

### MtrRole（Moetran 角色 ID）

与 Moetran 平台对接的固定角色字符串 ID，用于 `default_role` 字段：

- `"63d87c24b8bebd75ff934264"`：`ADMIN`（管理员）
- `"63d87c24b8bebd75ff934265"`：`PRINCIPAL`（负责人）
- `"63d87c24b8bebd75ff934266"`：`PROOFREADER`（校对）
- `"63d87c24b8bebd75ff934267"`：`TRANSLATOR`（翻译）
- `"63d87c24b8bebd75ff934268"`：`TYPESETTER`（排版）
- `"63d87c24b8bebd75ff934269"`：`INTERN`（实习）

### 语言代码

推荐使用的语言代码（用于 `source_language` / `target_languages`）：

- `"ja"`：日语
- `"zh_CN"` 或 `"zh-CN"`：简体中文
- `"zh_TW"` 或 `"zh-TW"`：繁体中文
- `"ko"`：韩语
- `"en"`：英语

其他语言可按 Moetran 平台支持的语言代码扩展。

---

## 请求响应示例集合

### 示例：完整的创建项目流程

1. **创建项目集**

```bash
curl -X POST https://api.example.com/api/v1/projsets \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "projsetName":"主线剧情",
      "projsetDescription":"主线任务相关文本",
      "teamId":"team_a",
      "mtrToken":"<moetran-token>"
    }'
```

响应：

```json
{
    "code": 201,
    "data": {
        "projset_serial": 1
    }
}
```

2. **创建项目**

```bash
curl -X POST https://api.example.com/api/v1/projs \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "projName":"第一章对话",
      "projDescription":"第一章主角对话文本",
      "teamId":"team_a",
      "projsetId":"projset_123",
      "mtrAuth":"<moetran-token>",
      "sourceLanguage":"ja",
      "targetLanguages":["zh_CN"],
      "defaultRole":"63d87c24b8bebd75ff934267"
    }'
```

响应：

```json
{
    "code": 201,
    "data": {
        "proj_id": "proj_1",
        "proj_serial": 1,
        "projset_index": 1
    }
}
```

3. **指派成员到项目**

```bash
curl -X POST https://api.example.com/api/v1/projs/proj_1/assign \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "memberId":"member_1",
      "mtrAuth":"<moetran-token>",
      "isTranslator":true,
      "isProofreader":false,
      "isTypesetter":false
    }'
```

响应：

```json
{
    "code": 204,
    "data": null
}
```

4. **查询项目详情及成员**

```bash
curl -X POST https://api.example.com/api/v1/projs/search \
    -H "Authorization: Bearer <jwt>" \
    -H "Content-Type: application/json" \
    -d '{
      "proj_ids":["proj_1"]
    }'
```

响应：

```json
{
    "code": 200,
    "data": [
        {
            "proj_id": "proj_1",
            "proj_name": "【1-1】第一章对话",
            "description": "第一章主角对话文本",
            "projset_id": "projset_123",
            "projset_serial": 1,
            "projset_index": 1,
            "translating_status": 0,
            "proofreading_status": 0,
            "typesetting_status": 0,
            "reviewing_status": 0,
            "is_published": false,
            "members": [
                {
                    "member_id": "member_1",
                    "username": "alice",
                    "is_admin": false,
                    "is_translator": true,
                    "is_proofreader": false,
                    "is_typesetter": false,
                    "is_principal": true
                }
            ]
        }
    ]
}
```

---

## 可能的后续扩展

- 团队管理接口
- Token 刷新 & 注销机制
- 操作审计日志
- 项目评论/讨论功能
