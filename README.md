# README

```rust
#[encode(unit=byte)]
struct Ehdr {
    /// 无 len 属性，则占用长度：sizeof::<[u8;3]>()
    magic: [u8;3],
    offset: u32,

    /// flags1 占用 1 个字节
    #[len=1]
    flags1: Flags,

    /// flags2 占用 2 个字节，flags1 和 flags2 使用相同的编码方式，但是字节长度不同。
    #[len=2]
    flags2: Flags,
}

#[encode(unit=bit)]
struct Flags {
    #[pos(-1)]
    is_a: bool,

    #[pos(1)]
    is_b: bool,

    #[pos(2..=3)]
    is_c: bool,
}

struct Unsigned;

#[encode(unit=bit)]
struct S8 {
    #[pos(7)]
    sign:bool,
    #[pos(0..=6)]
    value: Unsigned,
}
=>

struct Flags {
    is_a: bool,
    is_b: bool,
    is_c: bool,
}

impl Flags {
    pub fn unpack(raw:u8) -> FlagsCache {
        
        }
}
impl Flags {
    pub fn pack(&self) -> u8 {

        }
}

```

我们将 rust 中的所有类型都视作 encode。
类型即编码。
`bool`: 所占用的内存空间，如果全为 0 则为 true，否则为 false。
但是我们有时候希望只有特定值为 true，或者特定值为 false。
由于 true = !false，所以只需要指定 true 的合法值即可。

```rust
#[encode(unit=u8)]
struct A {
    #[len = 2]
    #[true = {0,1,2,3}]
    field1: bool,
    #[len = 3]
    #[trans=field2_trans]
    field2: bool,
    #[len = 1]
    field3: Ver,
}

enum Ver {
    V1,
    V2,
}

impl TryFrom<u8> for Ver {

    fn try_from(value:u8)-> Result<Ver, &'static str>{

    }
}

fn field2_trans(raw: [u8;3]) -> bool {
    false
}

#[encode(uint=bit, within=u8)]
struct B {
    #[len = 2]
    #[trans=field1_trans]
    field1: bool,
    #[len = 1]
    field2: bool,
}

fn field1_trans(raw:u8) -> bool {
    match raw {
        0|1|2 => true,
        3 => false,
        _ => panic!("never here");
    }
}

#[auto_map]
enum V1 {
    #[disc(0..=2, 3..=4, 7, 9)]
    #[disc(11)]
    A1(u8),
    #[disc(1)]
    A2,
    #[disc(2)]
    A3,
}

```

数值也是符号，如 1 和 1.2 以及 0b11 0x12 都是符号，我们只是将其视作符合特别形式的符号。
这些符号我们不用事先定义。在实际编码中，我们通常会将数值符号映射到具有实在意义的符号。
这也是枚举类型存在的意义。

~~而编码是数值符号的组合。~~

原始数据有时候不一定合法，所以 unpack 可能需要返回 Result<>，这样一来就需有一个 Error:

Error::unpack_field1(usize, &'static str)，每一个编码都有自己的 Error。 unpack_field1.0 => offset, unpack_field1.1=>reason

问题是如何获取该 reason ?

数值符号化的过程中可能会出现非法值。有些值是未定义的，有些值是保留的，有些值是超出范围的。未定义表示当前行为不可预知，保留的表示未来行为不可预知。超出范围表示当下行为可知，但是不符合预期。

```rust
#[bitmap(u8)]
struct A {
    #[pos(0..=2)]
    #[may_fail]
    version: Ver,
    #[pos(3..=4)]
    #[never_fail]
    flag:Flags,
}
```

`Ver` 既可以是结构体也可以是枚举类型，或者是任何类型实现了 TryFrom 或 From trait 的类型。

如果 A 中的任何字段具有 may_fail 属性，则实现 `TryFrom<u8>` 接口。
如果均不具有 may_fail 则实现 `Fron<u8>` 接口。
A 同时会实现 `TryFrom<[u8]>` 接口，这样方便将 A 的置于其他结构体中。

默认是 may_fail，never_fail 需要显式指定。

```rust
#[bytemap([u8;10])]
struct A1 {
    #[pos(0..=2)]
    version: Ver,
    #[pos(3)]
    bitmaped_data: A,
}
```

`#[bytemap([u8; 10])]` 属性表示将十个字节的数组映射到结构体 `A1` 中。
A1 中的任何字段的类型必须实现 `TryFrom<[u8]>`

```rust
#[valuemap(u8)]
enum V1 {
    #[range(0..=2)]
    D1,
    #[range(3..=10)]
    D2,
    #[range(13..=90)]
    D3,
    #[all_other]
    D4,
    #[range(100..=109)]
    D5(u8),
    #[range(123..=144)]
    D6(C)
}
```

C 是复合类型。需要实现 `try_from(u8)`
