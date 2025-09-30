# LLVM vs GCC Investigation Results

## Summary
This document captures the findings from investigating LLVM backend compatibility with STM32F429I-DISCO embedded Rust development.

## Test Configuration
- **Hardware**: STM32F429I-DISCO board with LTDC display and MPU6050 I2C sensor
- **Application**: Tilt-responsive square (hardware timing-critical)
- **Rust Target**: `thumbv7em-none-eabihf`
- **Comparison**: LLVM (`rust-lld`) vs GCC (`arm-none-eabi-gcc`)

## Results

### ✅ Working Configurations

#### LLVM Backend (Final Working)
```toml
[target.thumbv7em-none-eabihf]
linker = "rust-lld"
rustflags = [
    "-C", "link-arg=-Tlink.x",
]
# opt-level=0 by default (no optimizations)
```
**Status**: ✅ FULLY FUNCTIONAL - Display works, tilt control responsive

#### GCC Backend (Baseline)
```toml
[target.thumbv7em-none-eabihf]
rustflags = [
    "-C", "link-arg=-Tlink.x",
]
# Uses arm-none-eabi-gcc linker by default
```
**Status**: ✅ FULLY FUNCTIONAL - Works with all optimization levels

### ❌ Failed Configurations (LLVM)

#### Basic Optimizations
```toml
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "opt-level=1",
]
```
**Status**: ❌ BLANK DISPLAY - Hardware initialization fails

#### Standard Optimizations  
```toml
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "opt-level=2",
]
```
**Status**: ❌ BLANK DISPLAY - Hardware initialization fails

#### Size Optimizations
```toml
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "opt-level=s",
]
```
**Status**: ❌ BLANK DISPLAY - Even conservative optimizations fail

#### Selective Optimization Disabling
```toml
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "opt-level=1",
    "-C", "no-vectorize-loops",
    "-C", "no-vectorize-slp",
]
```
**Status**: ❌ BLANK DISPLAY - Selective disabling insufficient

## Root Cause Analysis

### Memory-Mapped I/O Sensitivity
The STM32F429 LTDC (LCD Controller) and I2C peripheral initialization requires:
- Precise register write ordering
- Specific timing between operations  
- Preservation of volatile memory access patterns

### LLVM vs GCC Differences
1. **Optimization Philosophy**:
   - **GCC**: More conservative with embedded/volatile operations
   - **LLVM**: More aggressive optimization, even at basic levels

2. **Memory Access Handling**:
   - **GCC**: Respects volatile semantics more strictly
   - **LLVM**: May reorder or combine memory-mapped register accesses

3. **Instruction Scheduling**:
   - **GCC**: Preserves timing-critical sequences  
   - **LLVM**: More aggressive instruction reordering

## Implications

### For Embedded Rust Development
- **LLVM is viable** for embedded projects but requires careful configuration
- **Any optimization** with LLVM can break hardware timing-critical code
- **GCC remains more reliable** for complex hardware interaction scenarios

### Performance Trade-offs
- **LLVM + opt-level=0**: Larger binaries, but functional hardware
- **GCC + optimizations**: Smaller, faster binaries with reliable hardware support

## Recommendations

### Use LLVM When:
- Exploring different code generation strategies
- Taking advantage of LLVM-specific features
- Learning/experimenting with different toolchains
- Performance is less critical than functionality

### Use GCC When:  
- Optimized binaries are required
- Complex hardware timing is critical
- Production embedded systems
- Maximum compatibility is needed

### Best Practices
1. **Test thoroughly** when switching between LLVM and GCC
2. **Start with opt-level=0** when using LLVM for embedded
3. **Profile before optimizing** - measure actual performance impact
4. **Consider hybrid approach** - develop with LLVM, deploy with GCC

## Conclusion

Both LLVM and GCC are viable for embedded Rust, but they have different strengths:
- **LLVM**: Better for experimentation and learning, requires opt-level=0 for hardware compatibility
- **GCC**: Better for production embedded systems with optimization requirements

The key insight is that **compiler backend choice affects hardware interaction** in ways that go beyond simple performance differences.

---
*Investigation completed: September 30, 2025*  
*Hardware: STM32F429I-DISCO with LTDC + MPU6050*  
*Application: Tilt-responsive square with 1ms update loop*