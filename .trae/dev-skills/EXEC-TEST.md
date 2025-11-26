# 执行模块测试案例

1. 重启服务
    - 保证服务管理， pkill -f zentao-mcp-server
    - cargo run --bin zentao-mcp-server

2. 执行测试，检查测试结果，如果错误，验证
    - cargo run --bin test_[module]
    - 从日志最开始的错误开始，检查错误信息
    - 进行测试修正，或者修复模块代码
    - 修正过程中，除非必要，不要增加无关字段
    - 重启服务，继续测试

3. 重复步骤2，直到所有模块的测试案例都通过