
# Notes

- When you use `core::hint::black_box` with a function pointer, the compiler must materialize the pointer, which introduces `.rdata` and `.pdata` sections


# Fixes / Workarounds / Issues

[Incorrect build issue displayed: "found duplicate lang item panic_impl"
](https://github.com/rust-lang/rust-analyzer/issues/4490)

`settings.json`

```json
{
    "rust-analyzer.check.allTargets": false
}
```

```
cargo tree -e features --no-default-features
```

[]

C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64
https://hackyboiz.github.io/2025/05/12/ogu123/NamedPipe/EN/
https://github.com/wonderzdh/Self-Remapping-Code/blob/master/SelfRemappingCode/remap.cpp
https://github.com/Hendi48/Magicmida/blob/master/PEInfo.pas
https://wutils.com/wmi/root/cimv2/win32_systemdriver/#create_methods
https://gist.github.com/soxfmr/16c495d6e4ad99e9e46f5bfd558d152f
https://hexblog.com/wp-content/uploads/2012/06/Recon-2012-Skochinsky-Compiler-Internals.pdf#2#1
https://kymb0.github.io/SEH-primer/
http://uninformed.org/index.cgi?v=4&a=1&p=18
https://stackoverflow.com/questions/78933204/how-do-i-interpret-xdata-and-pdata-section-to-hand-write-runtime-function-and-un