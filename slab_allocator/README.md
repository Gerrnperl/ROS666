# Slab Allocator

☝🏻🤓 An drop-in replacement for [rcore-os/buddy_system_allocator](https://github.com/rcore-os/buddy_system_allocator). But it use slab allocator instead. 

## Reference

- 接口设计和 Mutex 实现参考 [rcore-os/rCore-Tutorial-v3](https://github.com/rcore-os/buddy_system_allocator) (MIT License)。
- Slab Allocator 实现参考 [slab_allocator - Slab allocator for no_std systems. ](https://github.com/weclaw1/slab_allocator/tree/master) (MIT License)。
  
依赖项:
- [phil-opp/linked-list-allocator](https://github.com/phil-opp/linked-list-allocator) (MIT License or Apache License 2.0)
- [mvdnes/spin-rs](https://github.com/mvdnes/spin-rs.git) (MIT License)。