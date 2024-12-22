# 用户应用程序

运行在系统用户态的应用程序。

通过 [user-lib](../user-lib) 提供的接口，用户应用程序可以调用系统调用，实现对系统资源的访问。

用户应用程序列表在 [Cargo.toml](Cargo.toml) 中 `[[bin]]` 和 `package.metadata.applications` 字段中定义。
