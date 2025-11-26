从代码提取模块服务配置 模块到 ZML 格式

已经完成配置案例 config/zml/

整体要求：
- 提取 模块到 config/zml/模块.zml
- 信息完全，不能多也不能少，具体要求见各个模块
- 提取完成后允许 cargo run --bin test_modular_loader 进行验证，有错误自行修复

命名规则, 必须遵守
- API 请求及响应参数 使用 camelCase 命名规范, 如果有特殊说明，如rename, 以其为准
- 文件命名必须符合 snake_case 规范
- 方法名必须符合 snake_case 规范
- 对象类型定义名称必须符合 snake_case 规范
