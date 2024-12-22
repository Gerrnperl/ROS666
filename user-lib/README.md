# 用户程序库

<!-- Provides access to system functions for user applications. -->

用户程序库，用于在用户程序中调用系统功能。

在无`std`的情况下，用户程序需要调用系统功能，例如`println!`、`exit`等，此时用户程序需要调用系统库。

此库提供了系统功能的实现，用户程序以此作为依赖，以调用系统功能。