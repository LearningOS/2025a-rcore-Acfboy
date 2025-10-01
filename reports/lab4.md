## 总结

实现了文件系统相关的 linkat, unlinkat, fstat 系统调用。

## 问答

root_inode 作为文件的根目录，存了所有文件的 dirent，如果损坏，我们将无法找到文件对应的 inode。