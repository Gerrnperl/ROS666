# 文件系统镜像打包工具

用于将应用程序打包成文件系统镜像，以便于加载到系统内核中。

## 使用方法

```text
Usage: ros-fs-fuse [OPTIONS] --target <TARGET> --output <OUTPUT>
Options:
  -a, --app <APP>...     应用程序名称列表，以空格分隔
  -t, --target <TARGET>  目标目录，存放应用程序二进制文件
  -o, --output <OUTPUT>  文件系统镜像输出路径
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```
