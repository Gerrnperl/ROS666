# 存储管理模块概述

## 模块简介

存储管理模块是操作系统内核的重要组成部分，其主要职责是为操作系统提供高效、稳定的内存和存储管理能力。在基于 **RISC-V** 架构的系统中，存储管理模块实现了**虚拟内存地址空间的抽象**、**物理页帧的分配与管理**、**内核动态内存分配**以及**多级页表的维护与操作**。通过这些功能，存储管理模块为操作系统提供了可靠的存储支持，同时提升了内存管理的灵活性和效率。

本模块设计的核心思想是利用**分层结构**和**模块化**实现，将存储管理划分为多个子功能模块，每个模块分别负责特定的任务，并通过统一的接口协同工作。通过后续的介绍我们容易得知这种设计方式不仅仅提升了本模块的可维护性和扩展性，更重要的是**代码逻辑变得更加清晰**、**使得用户阅读起来更加容易**。

---

## 模块的核心作用

1. **虚拟地址空间管理**  
   通过分页机制为每个进程提供独立的虚拟地址空间，隔离不同进程之间的内存访问，确保系统的安全性和稳定性。

2. **物理页帧管理**  
   对物理内存页帧的分配和回收进行精确控制，为内核和用户进程提供可靠的内存分配接口。

3. **动态内存分配**  
   实现内核级的动态内存分配器，用于满足内核在运行时的内存需求，支持高效的内存分配与回收。

4. **多级页表管理**  
   结合硬件支持的多级页表机制，实现虚拟地址到物理地址的高效映射，提供虚实地址转换的基础支持。

---

## 模块结构

存储管理模块由以下几个核心部分组成，每个部分独立负责特定的功能，并通过接口进行交互：

1. **地址抽象与管理（`address.rs`）**  
   提供对虚拟地址和物理地址的抽象和操作方法，是模块中所有内存管理功能的基础。

2. **物理页帧分配器（`frame_allocator.rs`）**  
   管理物理内存的使用情况，提供页帧的分配与回收功能，为内核和用户进程提供物理内存支持。

3. **动态内存分配器（`heap_allocator.rs`）**  
   负责内核运行时动态内存的分配与回收，通过堆分配策略实现高效的内存使用。

4. **多级页表管理（`page_table.rs`）**  
   实现基于 RISC-V SV39 模式的多级页表管理，支持虚拟地址和物理地址的映射与权限控制。

5. **地址空间管理（`memory_set.rs`）**  
   定义地址空间的结构与管理逻辑，包括逻辑段（segment）的划分和多级页表的管理。

6. **模块初始化（`init.rs`）**  
   负责存储管理模块的初始化，包括页帧分配器、堆分配器和地址空间的初始化。

---

## 模块的设计目标

1. **安全性**  
   通过独立的虚拟地址空间和权限控制机制，确保内核与用户进程、进程与进程之间的内存隔离。

2. **高效性**  
   利用缓存和分页机制，减少内存访问的延迟，同时优化物理内存的利用率。

3. **灵活性**  
   提供动态内存分配支持，允许内核在运行时根据需要分配或释放内存资源。

4. **可扩展性**  
   通过模块化设计和统一接口，方便未来的功能扩展和代码维护。

---

## 模块在操作系统中的地位

存储管理模块位于操作系统内核的核心位置，作为桥梁连接内核的其他模块与硬件设备：

- **向上**：为任务调度模块、文件系统模块等提供内存分配和地址管理支持。
- **向下**：通过多级页表和物理页帧分配器与硬件的内存管理单元（MMU）进行交互，完成地址转换和内存分配。

通过这一模块，操作系统能够屏蔽底层物理内存的复杂性，为用户程序提供统一且高效的内存管理接口。

---

存储管理模块 - 动态内存分配（Slab 堆分配器）
=======================================

简介
----

在没有动态内存分配的操作系统中，内核和用户程序只能使用静态内存分配，即在编译时确定内存大小。例如，为了存放一个数组，需要提前分配足够的内存空间。

```rust
let mut array = [0; 1024]; // 静态分配 1024 个元素的数组
```

但是，这个方法存在以下问题：
- 静态内存分配需要提前确定内存大小，不适用于动态数据结构。
- 静态内存分配可能导致内存浪费或溢出。例如，这个数组可能只在极少数情况下才被使用，或者只在极端情况下才需要 1024 个元素，但是却占用了 1024 个元素的内存空间。

为了解决这些问题，操作系统需要提供动态内存分配的支持。

动态内存分配允许程序在运行时根据需要分配和释放内存，提高内存利用率和灵活性。

在本项目中，动态内存分配器在 `slab_allocator` 包中实现，主要采用 Slab 分配器的设计。Slab 分配器是一种高效的内存分配器，通过预先分配一定数量的固定大小的内存块（Slab），并在需要时分配和回收这些内存块，以减少内存碎片和提高性能。

本节将详细分析堆分配器的设计与实现，并通过代码解析堆分配的核心功能。

动态内存分配的基础概念
----------------------

### 堆内存

堆内存是在内核和用户态程序执行时动态分配的内存区域，其实际使用大小随程序运行时的需要而变化。

对于一个需要使用堆内存的变量，其在生命周期开始时，需要向堆分配器请求一块内存；在生命周期结束时，需要释放这块内存。

在此内核和用户程序库的实现中，堆内存是一块静态分配的内存区域的包装和抽象，在这块静态内存区域上实现堆分配器，即可将这块静态内存区域的子区域作为申请的内存块，实现内存的动态分配。

### 堆分配器

堆分配器是一个管理堆内存的模块，负责分配和释放内存。一个堆分配器在其初始化时即*拥有*了一块静态内存区域，可以在这块静态内存区域上进行内存的动态分配。

堆分配器的设计目标：

1.  **高效性**：减少分配与释放的时间开销。

2.  **低碎片率**：尽可能减少内存碎片。

3.  **线程安全**：支持并发环境中的内存分配。

此内核和用户程序库的堆分配器采用 Slab 分配器的设计，通过预先分配一定数量的固定大小的内存块（Slab），并在需要时分配和回收这些内存块，以减少内存碎片和提高性能。

而对于大块的内存分配，堆分配器转用 LinkedList 分配器，通过链表的方式管理大块内存的分配与释放。

如此，便可实现了一个高效、低碎片率的堆分配器，对于小内存分配，其不会产生较大的内存碎片，对于大内存分配，其也能够高效地进行内存分配。

#### Slab 分配器

Slab 分配器是一种简单的内存分配器，它将内存分配为固定大小的块，并在需要时分配这些块。

Slab 分配器的核心思想是将内存划分为多个固定大小的块，每个块称为一个 Slab。Slab 分配器维护一个空闲链表，用于存储空闲的 Slab。当需要分配内存时，Slab 分配器从空闲链表中取出一个 Slab，并将其分配给请求者。当释放内存时，Slab 分配器将 Slab 放回空闲链表。

Slab 分配器的优点是高效、低碎片率，适用于分配固定大小的内存块。

```rust
pub struct Slab {
    pub block_size: usize,
    pub block_num: usize,
    free_list: FreeList,
}

struct FreeList {
    len: usize,
    head: Option<&'static mut FreeBlock>,
}

struct FreeBlock {
    next: Option<&'static mut FreeBlock>,
}
```

每个空闲链表链表项 `FreeBlock` 结构体“占用”一个固定大小的内存块，并指向下一个空闲块。
一个空闲链表即可将同种大小的空闲块串联起来，

在分配时，只需从链表中取出一个空闲块; 在释放时，只需将空闲块插入链表头部即可。

```rust
pub fn push(&mut self, free_block: &'static mut FreeBlock) {
    free_block.next = self.head.take();
    self.head = Some(free_block);
    self.len += 1;
}

pub fn pop(&mut self) -> Option<&'static mut FreeBlock> {
    self.head.take().map(|free_block| {
        self.head = free_block.next.take();
        self.len -= 1;
        free_block
    })
}
```

在扩充 Slab 分配器时，只需要在新分配的内存区域上以 Slab 的大小建立一个新的空闲链表，并将其加入到 Slab 分配器的链表中即可。
```rust
pub fn grow(&mut self, start: Address, slab_size: usize) {
    let block_num = slab_size / self.block_size;
    self.block_num += block_num;
    // 添加到 self.free_list
    for i in 0..block_num {
        let block = (start + i * self.block_size) as *mut FreeBlock;
        let block = unsafe { &mut *block };
        self.free_list.push(block);
    }
}
```

#### 堆分配器核心定义

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ rust
pub struct Heap {
    /// Slab 分配器数组
    slabs: [slab::Slab; SLABS_NUM],
    /// Fallback Linked List Allocator
    fallback: linked_list_allocator::Heap,
    /// 用户请求的字节数
    user: usize,
    /// 实际分配的字节数
    allocated: usize,
    /// 堆中的总字节数
    total: usize,
}
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

此堆分配器包含了两种内存分配器：Slab 分配器和 LinkedList 分配器。

Slab 分配器用于分配小块内存，在这里，我们定义了 `SLABS_NUM` (=7) 种不同大小的 Slab 分配器，分别用于分配 64、128、256、512、1024、2048 和 4096 字节大小的内存块。

LinkedList 分配器用于分配大块内存，当 Slab 分配器无法满足用户请求时，堆分配器会转用 LinkedList 分配器。

#### 堆分配器初始化

在堆分配器构建时，其各个子分配器都是空的，在初始化时，需要将一段连续的内存空间划分给各个子分配器。

```rust
pub unsafe fn init(&mut self, start: usize, size: usize) {
    let alloc_size = size / ALLOCATORS_NUM;
    let total_slab_size = alloc_size * SLABS_NUM;
    let fallback_size = alloc_size;
    unsafe {
        self.add_to_heap(start, start + total_slab_size);
        self.init_fallback(start + total_slab_size, fallback_size);
    }
}
```

在初始化时，堆内存分配器*以相同的权重*将初始内存分配给各个子分配器，即确保每个子分配器管理的内存尽可能相等，以满足不同大小内存块的分配需求。

```rust
unsafe fn init_fallback(&mut self, mut start: usize, size: usize) {
    start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
    unsafe {
        self.fallback.init(start as *mut u8, size);
    }
    self.total += size;
}
pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
    start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
    end &= !size_of::<usize>() + 1;
    assert!(start <= end);
    let new_heap_size = end - start;
    self.total += new_heap_size;
    let slab_size = new_heap_size / SLABS_NUM;
    for slab_i in 0..SLABS_NUM {
        let slab_start = start + slab_i * slab_size;
        match self.get_inner(slab_i) {
            AllocType::Slab(i) => {
                self.slabs[i].grow(slab_start, slab_size);
            }
            AllocType::Fallback => {
                unreachable!("Fallback allocator should not be initialized here");
            }
        }
    }
}
```

这里，`add_to_heap` 方法将一段连续的内存空间划分给各个 Slab 分配器，在分配时，将内存空间平分给各个 Slab 分配器，每个 Slab 分配器获得 从总内存起始`start` + `i * slab_size` 到 `start` + `(i+1) * slab_size` 的内存空间。

由于 Linked List 分配器的内存分配方式不同，其需要连续的内存空间，因此在初始化时，需要将一段连续的内存空间划分给 Linked List 分配器。在这之后不随 Slab 分配器的内存分配而变化。

#### 分配

堆分配器的分配方法是一个简单的分配器选择方法，它根据用户请求的内存大小选择合适的 Slab 分配器或 Linked List 分配器进行内存分配。

```rust
pub fn get_slab_index(mut size: usize) -> usize {
    if size <= MIN_ALLOC_SIZE {
        return 0;
    }
    if size > MAX_SLAB_SIZE {
        return ALLOCATORS_NUM - 1; // 回退到链表分配器
    }
    // 64 -> 0, 65-128 -> 1, 129-256 -> 2, ...
    size = size.next_power_of_two();
    size.trailing_zeros() as usize - 6
}
```

`get_slab_index` 方法根据用户请求的内存大小选择合适的 Slab 分配器，如果请求的内存大小小于等于最小内存块大小，则选择第一个 Slab 分配器；如果请求的内存大小大于最大 Slab 分配器的内存块大小，则选择 Linked List 分配器；否则，选择最接近且大于等于请求内存大小的 Slab 分配器。

```rust
pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
    let size = layout.size();
    match self.select_allocator(&layout) {
        AllocType::Slab(i) => {
            let ret = self.slabs[i].alloc();
            if ret.is_ok() {
                self.user += size;
                self.allocated += self.slabs[i].block_size;
            }
            ret
        }
        AllocType::Fallback => {
            let ret = self.fallback.allocate_first_fit(layout);
            if ret.is_ok() {
                self.user += size;
                self.allocated += size;
                Ok(ret.unwrap())
            } else {
                Err(AllocError)
            }
        }
    }
}
```

`alloc` 方法根据用户请求的内存大小选择合适的分配器，并调用相应的分配方法进行内存分配。如果分配成功，则更新用户请求的字节数和实际分配的字节数。分配操作会返回一个非空的内存指针，或者返回分配错误。

#### 释放

和分配类似，堆分配器的释放方法也是一个简单的释放器选择方法，它根据用户请求的内存大小选择合适的 Slab 分配器或 Linked List 分配器进行内存释放。

```rust
pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
    let size = layout.size();
    match self.select_allocator(&layout) {
        AllocType::Slab(i) => {
            self.slabs[i].dealloc(ptr);
            self.user -= size;
            self.allocated -= self.slabs[i].block_size;
        }
        AllocType::Fallback => {
            unsafe { self.fallback.deallocate(ptr, layout) };
            self.user -= size;
            self.allocated -= size;
        }
    }
}
```

`dealloc` 方法根据用户请求的内存大小选择合适的分配器，并调用相应的释放方法进行内存释放。如果释放成功，则更新用户请求的字节数和实际分配的字节数。

#### LockedHeap

堆内存分配器不只运行在内核态，也运行在用户态。在用户态，堆内存分配器需要支持并发环墐，因此需要加锁以保证线程安全。

LockedHeap 是一个对 Heap 的封装，它在堆内存分配器的基础上增加了互斥锁，以保证堆内存分配器的线程安全性。

```rust
#[cfg(feature = "use_spin")]
pub struct LockedHeap(Mutex<Heap>);
```

在使用 LockedHeap 时，需要先使用 `.lock()` 方法获取互斥锁，然后再调用堆内存分配器的方法。

#### 注册

堆内存分配器需要在内核启动时注册，以便内核和用户程序能够使用堆内存分配器。

注册堆内存分配器首先需要为其实现 Rust alloc 库的 `GlobalAlloc` trait，这样，Rust 的内存分配器就能够使用堆内存分配器进行内存分配，从而可以在内核和用户程序中使用堆内存分配器及构建在其之上的 动态数据结构，如 `Vec`、`Box` 等。

```rust
#[cfg(feature = "use_spin")]
unsafe impl alloc::GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc(layout).ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock()
            .dealloc(unsafe { NonNull::new_unchecked(ptr) }, layout)
    }
}

实现时，只需要简单地调用堆内存分配器的 `alloc` 和 `dealloc` 方法即可。

实现该 trait 后，即可向 Rust 的内存分配器注册堆内存分配器，使得 Rust 的内存分配器能够使用堆内存分配器进行内存分配。

我们需要定义一个全局的 `HEAP_ALLOCATOR` 静态变量，通过 `#[global_allocator]` 属性将其注册为 Rust 的全局内存分配器。

最后，将一块静态内存空间划分给堆内存分配器，并初始化堆内存分配器。

```rust
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[allow(static_mut_refs)]
/// 初始化内核堆内存分配器
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.lock()
            .init(HEAP.as_ptr() as usize, HEAP_SIZE);
    }
}
```

同时，我们也需要处理分配失败的情况，例如内存不足等。使用 `#[alloc_error_handler]` 属性，我们可以定义一个全局的内存分配错误处理函数，当内存分配失败时，Rust 的内存分配器会调用该函数。

```rust
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error: {:?}", layout)
}
```

小结
----

堆分配器是一个管理堆内存的模块，负责分配和释放内存。堆分配器通过 Slab 分配器和 Linked List 分配器实现内存的动态分配，提高内存利用率和灵活性。

在本节中，我们详细介绍了堆分配器的设计与实现，包括 Slab 分配器的设计思想、核心功能和代码实现。通过堆分配器，操作系统可以支持动态内存分配，为内核和用户程序提供强大的内存管理能力。

---
# 存储管理模块 - 总结

## 模块的重要性

存储管理模块是操作系统内核的核心组成部分，负责管理物理内存、虚拟地址空间以及动态内存分配。它通过模块化和分层设计实现了复杂功能的清晰分工，为操作系统提供了高效、稳定和灵活的存储支持。

本项目的存储管理模块涵盖了以下关键部分：
- **地址抽象与管理**：提供虚拟地址和物理地址的抽象与操作方法，构建了整个内存管理的基础。
- **物理页帧管理**：使用位图数据结构高效跟踪和管理物理页帧，提供了可靠的分配和回收机制。
- **多级页表管理**：基于 RISC-V SV39 分页机制，支持虚拟地址到物理地址的高效映射和权限控制。
- **地址空间管理**：通过逻辑段和多级页表的结合，实现虚拟地址空间的灵活组织与管理。
- **堆分配器设计**：基于 Slab 和 LinkedList 分配器，实现了高效、低碎片率的动态内存分配。

---

## 核心设计特点

1. **模块化设计**  
   - 各子模块（如地址管理、页表管理、地址空间管理）独立实现，易于扩展和维护。
   - 提供统一的接口，实现内存管理功能的松耦合。

2. **高效性**  
   - 使用位图和多级页表等高效的数据结构，优化了内存管理的性能。
   - 动态内存分配的线性分配策略保证了分配的快速性。

3. **安全性**  
   - 地址空间的权限控制和进程间的内存隔离确保了系统的稳定性和安全性。

4. **灵活性**  
   - 地址空间支持动态调整，满足不同任务对内存的需求。
   - 堆分配器为内核提供了按需分配的能力，适应运行时的多样化需求。

---

## 学习与实现的价值

本存储管理模块的实现为操作系统开发提供了以下学习和实践价值：

1. **理论与实践结合**  
   - 理解内存管理的核心概念（如地址映射、多级页表、逻辑段）。
   - 将理论知识（如分页机制）应用于实际代码开发。

2. **代码实现与优化**  
   - 学习如何使用数据结构（如位图、数组）高效管理资源。
   - 掌握内核级内存管理的实现方法与优化技巧。

3. **模块化设计思想**  
   - 提升对模块化设计和接口定义的理解，为大型系统的开发打下基础。

---

## 结束语

存储管理模块为操作系统的正常运行提供了强有力的支持，是操作系统开发的核心环节之一。本项目的实现涵盖了从地址抽象到动态分配的完整功能链，为进一步的学习和开发奠定了坚实的基础。

通过模块化设计和代码实践，我们不仅能够更好地理解存储管理的内部机制，还可以为实际操作系统的开发积累宝贵经验因而后续我们还将继续在本基础上进行改善。
