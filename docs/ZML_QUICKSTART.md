# ZML 快速上手（当前实现）

> 所有示例中的注释均为英文，符合工作区规则。

## 工具
- `zml`: 统一 ZML CLI（支持列出模块与编译为 JSON）

### 使用方式
- 列出模块（使用默认配置目录）：
```bash
cargo run --bin zml -- list
```
- 列出模块（指定自定义ZML目录）：
```bash
cargo run --bin zml -- list --dir custom/zml/dir
```
- 从文件读取并输出 JSON（使用默认配置目录）：
```bash
cargo run --bin zml -- compile -i config/zml/project.zml > project.json
```
- 从标准输入读取并输出 JSON：
```bash
cat config/zml/project.zml | cargo run --bin zml -- compile -- > project.json
```
- 美化输出（缩进）：
```bash
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```

### 参数说明
- `-i, --input FILE`: 输入的 ZML 文件路径；不提供时从 STDIN 读取
- `-p, --pretty`: 美化输出为缩进 JSON
- `-c, --config-dir DIR`: 配置目录路径（默认：自动检测程序目录下的config/）
- `-d, --dir DIR`: ZML 目录路径（默认：配置目录下的zml/）
- `-o, --out DIR`: 预设输出目录（默认：配置目录下的presets/）

## 示例：编译一个模块
```zml
module project {
    version: "1.0.0"
    description: "Project service"
    enabled: true
    access_level: public

    type project_user {
        id: integer                 // User ID
        account: string             // User account
    }

    method get_project_list {
        description: "Get project list"
        http_method: GET
        uri: "projects"
        access_level: public
        rate_limit: 50/60

        params {
            page: integer = 1       // Default 1
            limit: integer = 20     // Default 20
        }

        response: object {
            page: integer
            total: integer
        }
    }
}
```

编译：
```bash
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```

## 常见问题
- 空输入：从 STDIN 读取时，若内容为空，工具会报错并退出。
- 语法错误：编译器会输出错误信息；请根据 `docs/ZML_SPECIFICATION.md` 修复。
- 模板与继承：当前仅解析，不参与编译输出；请参考规范文档的限制章节。

## 进一步阅读
- 规范（中文）：`docs/ZML_SPECIFICATION.md`
- 规范（英文）：`docs/ZML_SPECIFICATION_EN.md`
- 嵌套与引用验证：`docs/ZML_NESTING_REFERENCE_VALIDATION*.md`