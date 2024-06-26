<img src="./docs/.gitbook/assets/ola.jpg" alt="Ola Logo" style="align:center" />

 [![CI](https://img.shields.io/github/actions/workflow/status/Sin7y/ola-lang/release.yml)](https://github.com/Sin7Y/ola-lang/actions)
 [![Project license](https://img.shields.io/github/license/Sin7y/ola-lang)](LICENSE)
 ![Lines of Code](https://aschey.tech/tokei/github/Sin7y/ola-lang?category=code)
 [![Twitter](https://img.shields.io/twitter/follow/ola_zkzkvm?style=social)](https://twitter.com/ola_zkzkvm)

 ## Introduction

Ola is a high-level programming language for developing OlaVM smart contracts. It is Turing complete and can be used to write arithmetic programs. The computing process is proven by the OlaVM back-end proof system, which verifies that the OlaVM processing is accurate. Most of the existing programming languages in the ZKP field require fundamental knowledge of the circuit field, which is not universal, or the execution process is difficult to be proven and verified by ZKP.

## 👉👉📖 [Ola Language Documentation](https://olang.gitbook.io/ola-lang/)

## Simple Example

The following shows a simple contract for calculating the Fibonacci function

```
contract Fibonacci {

    fn fib_recursive(u32 n) -> (u32) {
        if (n <= 2) {
            return 1;
        }
        return fib_recursive(n -1) + fib_recursive(n -2);
    }

    fn fib_non_recursive(u32 n) -> (u32) {
        u32 first = 0;
        u32 second = 1;
        u32 third = 1;
        for (u32 i = 2; i <= n; i++) {
             third = first + second;
             first = second;
             second = third;
        }
        return third;
    }

}
```

## Ola language features

### Language Type 

- **Simple Types**: Includes `u32`, `field`, `bool`, `enum`, `address`, and `hash` types. In Ola, all data types are internally represented as combinations of the `field` type. The maximum value for the `field` type is `0xFFFFFFFF00000001`. The `address` and `hash` types are internally represented as arrays of four `field` elements.
- **Complex Types**: Includes static arrays, dynamic arrays, `string`, `fields`, structures, and mappings. The `fields` type is a dynamic array of `field` elements.
- **Type Aliases**: Support for type aliases is included.

### Operators

- Different types support different operators. The `u32` type supports the most operators, such as: `+`, `-`, `*`, `/`, `%`, `**`, `==`, `!=`, etc.

### Functions

- Functions are supported in a manner similar to Solidity and Rust, with the keyword `fn` used to declare functions.

### Control Flow

- Control flow constructs include `if` statements, `if-else` statements, `while` statements, `do-while` statements, `for` loops, `continue`, and `break`.

### Prophet Functions

- The `prophet` functions in Ola language are used for non-deterministic computations in specific scenarios, such as `u32_sqrt`, `u32_quick_sort`, etc.

### Core Lib Functions

* The ola language provides many core libraries, including `assert` ,`print` `fileds_conct`, `encode` `decode`, and so on.

### IR Generation

- The Ola language is processed by the front-end into a subset of LLVM IR for compilation.

## Olac Back-End

The LLVM IR subset generated by the Olac compiler front-end is downgraded to OlaVM bytecode for execution by the OlaVM virtual machine.

### Supported LLVM IR Subset

#### Type System:

- `void` type
- Function type
- Single-value types within the first-order type support `i64`, `i1`, `ptr` types
- `label` and `token` types
- Aggregate types such as arrays and structures

#### Instruction Set:

- Terminal instructions: `ret`, `br`, `switch`
- Unary operation: `neg`
- Binary operations: `add`, `sub`, `mul`
- Logical operations: `and`, `or`, `xor`
- Aggregate operations: `insertvalue`, `extractvalue`
- Memory access and addressing: `alloca`, `store`, `load`, `getelementptr`
- Conversion instruction: `trunc`
- Other operations: `icmp`, `phi`, `call`

### Supported IR Libs Extensions

- Includes built-ins and prophets: `assert`, `rangecheck`, `u32_sqrt`, `div`, `mod`, `vec_new`, `poseidon_hash`
- Contract storage access operations: `set_storage`, `get_storage`
- Contract input/output access operations: `get_context_data`, `get_tape_data`, `set_tape_data`
- Debugging: `printf`
- Cross-contract calls: `contract_call`

## OlaVM Assembly

- The assembly language consists of the `program` and `prophet` parts, executed directly by the VM and interpreted by the embedded interpreter, respectively.
- `program` part: Defines the assembly output format, supporting the entire OlaVM custom instruction set.
- `prophet` part: Refers to the `prophets` set described in the IR libs extension.

## Future Work

- Introduction of a testing framework, more orderly language design, and robust compiler engineering implementation.
- Heap memory allocation and management reconstruction, more streamlined instruction generation.
- Optimization of code generation for front-end and back-end, higher efficiency in instruction generation and execution.
- Design of object-oriented syntax for contracts, enhancing the language's expressive power.
- Expansion of privacy features in contracts, supporting ZK privacy contract programming.

## 🧰 Troubleshooting
If you are having trouble installing and using Ola,  please open an [issue](https://github.com/Sin7Y/ola-lang/issues/new).

## License

[Apache 2.0](LICENSE)
