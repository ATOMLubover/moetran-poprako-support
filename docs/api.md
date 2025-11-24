# PopRaKo API 文档

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
    "message": "Invalid password"
}
```

HTTP 状态码与 `code` 字段保持一致（便于 API 网关或客户端根据任一来源判断）。

认证：除同步接口外，其余接口都需要在 Header 中携带：

```text
Authorization: Bearer <JWT>
```

JWT `sub` 字段即用户 `user_id`。

---

## User 部分

### 1. 同步用户信息

用于将外部（尨译）用户数据同步到本服务数据库，如果用户已存在则校验密码并返回新 token；如果不存在则创建后返回 token。

| 方法 | 路径                | 认证   | 幂等性                                          |
| ---- | ------------------- | ------ | ----------------------------------------------- |
| POST | `/api/v1/user/sync` | 不需要 | 幂等（同一 user_id + 正确密码重复请求返回 200） |

请求体（JSON）：

```json
{
    "user_id": "user_123",      // 唯一用户标识
    "username": "alice",        // 用户名（可与 user_id 相同）
    "email": "alice@example.com",
    "password": "plaintext"     // 明文密码（传输时需 HTTPS）
}
```

成功响应：

```json
// 已存在用户且密码正确 => 200
{
    "code": 200,
    "data": { "token": "<jwt-token>" }
}

// 新建用户 => 201
{
    "code": 201,
    "data": { "token": "<jwt-token>" }
}
```

错误响应：

| 场景           | code | message               |
| -------------- | ---- | --------------------- |
| 密码错误       | 401  | Invalid password      |
| JSON 解析错误  | 422  | Unprocessable entity  |
| 服务端内部错误 | 500  | Internal server error |

示例 cURL：

```bash
curl -X POST https://api.poprako.example/api/v1/user/sync \
    -H "Content-Type: application/json" \
    -d '{"user_id":"user_123","username":"alice","email":"alice@example.com","password":"secret"}'
```

### 2. 获取用户信息

返回用户的基本资料以及其所属的汉化组列表。

| 方法 | 路径                | 认证 |
| ---- | ------------------- | ---- |
| GET  | `/api/v1/user/info` | 需要 |

成功响应：

```json
{
    "code": 200,
    "data": {
        "user_id": "user_123",
        "username": "alice",
        "email": "alice@example.com",
        "teams": [
            { "team_id": "team_a", "team_name": "A组" },
            { "team_id": "team_b", "team_name": "B组" }
        ]
    }
}
```

可能的错误：

| 场景                               | code |
| ---------------------------------- | ---- |
| 未认证/Token非法                   | 401  |
| 用户不存在（理论上同步后才会调用） | 404  |
| 内部错误                           | 500  |

示例 cURL：

```bash
curl -H "Authorization: Bearer <jwt>" \
    https://api.poprako.example/api/v1/user/info
```

---

## Member 部分

### 3. 获取自己在特定汉化组中的成员信息

根据当前登录用户与指定 `team_id`，返回该用户在该汉化组的角色标记。若该团队不存在或用户不在团队中，则可能返回 404。

| 方法 | 路径                  | 认证 | 查询参数           |
| ---- | --------------------- | ---- | ------------------ |
| GET  | `/api/v1/member/info` | 需要 | `team_id=<string>` |

cURL 调用示例：

```bash
curl -G https://api.poprako.example/api/v1/member/info \
    -H "Authorization: Bearer <jwt>" \
    --data-urlencode "team_id=team_a"
```

成功响应：

```json
{
    "code": 200,
    "data": {
        "member_id": "member_789",
        "is_admin": false,
        "is_translator": true,
        "is_proofreader": false,
        "is_typesetter": false,
        "is_principal": false
    }
}
```

错误响应：

| 场景                       | code | 说明                   |
| -------------------------- | ---- | ---------------------- |
| 未认证                     | 401  | 缺失或非法 JWT         |
| team_id 缺失或格式错误     | 422  | 参数无法解析           |
| 团队不存在或用户不在该团队 | 404  | 未找到对应成员记录     |
| 内部错误                   | 500  | 一般为数据库或服务异常 |

---

## 公共错误语义

| code | 含义                               |
| ---- | ---------------------------------- |
| 200  | 成功                               |
| 201  | 已创建                             |
| 400  | 请求格式错误                       |
| 401  | 未认证或凭证无效                   |
| 404  | 资源不存在                         |
| 422  | 语义/字段解析失败（JSON 或 Query） |
| 500  | 服务器内部错误                     |

统一约定：

- 所有失败响应都包含 `message` 字段；
- 成功响应无 `message`（或忽略）；
- 列表等复杂接口今后会在 `data` 中扩展分页字段（如 `items`, `total`, `page`, `size`）。

---

## 后续扩展计划（占位）

| 模块  | 可能新增接口                     |
| ----- | -------------------------------- |
| Team  | 创建团队、列出团队、团队成员管理 |
| Auth  | Token 刷新、注销（黑名单机制）   |
| Audit | 操作日志查询                     |

以上接口补充后应继续在本文件中以同样格式维护。
