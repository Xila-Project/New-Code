#include "../../../ABI/include/Xila.h"
#include "platform_api_vmcore.h"

/****************************************************
 *                     Section 1                    *
 *        Interfaces required by the runtime        *
 ****************************************************/

/**
 * Initialize the platform internal resources if needed,
 * this function is called by wasm_runtime_init() and
 * wasm_runtime_full_init()
 *
 * @return 0 if success
 */
int bh_platform_init(void)
{
    return 0; // No initialization needed
}

/**
 * Destroy the platform internal resources if needed,
 * this function is called by wasm_runtime_destroy()
 */
void bh_platform_destroy(void) {}

/**
 ******** memory allocator APIs **********
 */

void *os_malloc(unsigned size)
{
    return Xila_memory_allocate(size);
}

void *os_realloc(void *ptr, unsigned size)
{
    return Xila_memory_reallocate(ptr, size);
}

void os_free(void *ptr)
{
    Xila_memory_deallocate(ptr);
}

/**omefile) will trigger cmake to rerun if that file changes and you're building s
 * Note: the above APIs can simply return NULL if wasm runtime
 *       isn't initialized with Alloc_With_System_Allocator.
 *       Refer to wasm_runtime_full_init().
 */

int os_printf(const char *format, ...)
{
    printf("os_printf: \n");

    int ret = 0;
    va_list args;
    va_start(args, format);
#ifndef BH_VPRINTF
    ret += vprintf(format, args);
#else
    ret += BH_VPRINTF(format, args);
#endif
    va_end(args);
    return ret;
}

int os_vprintf(const char *format, va_list ap)
{
    printf("os_vprintf: \n");

#ifndef BH_VPRINTF
    return vprintf(format, ap);
#else
    return BH_VPRINTF(format, ap);
#endif
}

/**
 * Get microseconds after boot.
 */
uint64 os_time_get_boot_us(void)
{
    return Xila_time_get_time_since_startup_microseconds();
}

/**
 * Get thread-specific CPU-time clock in microseconds
 */
uint64 os_time_thread_cputime_us(void)
{
    return Xila_get_cpu_time_microseconds();
}

/**
 * Get current thread id.
 * Implementation optional: Used by runtime for logging only.
 */
korp_tid os_self_thread(void)
{
    return Xila_get_current_thread_identifier();
}

/**
 * Get current thread's stack boundary address, used for runtime
 * to check the native stack overflow. Return NULL if it is not
 * easy to implement, but may have potential issue.
 */
uint8 *os_thread_get_stack_boundary(void)
{
    return Xila_get_thread_stack_boundary();
}

/**
 * Set whether the MAP_JIT region write protection is enabled for this thread.
 * Pass true to make the region executable, false to make it writable.
 */
void os_thread_jit_write_protect_np(bool enabled)
{
    // Not required
}

/**
 ************** mutext APIs ***********
 *  vmcore:  Not required until pthread is supported by runtime
 *  app-mgr: Must be implemented
 */

int os_mutex_init(korp_mutex *mutex)
{
    if (Xila_initialize_mutex(mutex))
        return 0;

    return 1;
}

int os_mutex_destroy(korp_mutex *mutex)
{
    if (Xila_destroy_mutex(mutex))
        return 0;

    return 1;
}

int os_mutex_lock(korp_mutex *mutex)
{
    if (Xila_lock_mutex(mutex))
        return 0;

    return 1;
}

int os_mutex_unlock(korp_mutex *mutex)
{
    if (Xila_unlock_mutex(mutex))
        return 0;

    return 1;
}

/**************************************************
 *                    Section 2                   *
 *            APIs required by WAMR AOT           *
 **************************************************/

Xila_memory_protection_type To_xila_memory_protection(int prot)
{
    Xila_memory_protection_type Xila_protection = 0;

    if (prot & MMAP_PROT_EXEC)
        Xila_protection |= Xila_memory_protection_execute;

    if (prot & MMAP_PROT_READ)
        Xila_protection |= Xila_memory_protection_read;

    if (prot & MMAP_PROT_WRITE)
        Xila_protection |= Xila_memory_protection_write;

    return Xila_protection;
}

Xila_memory_flag_type To_xila_memory_flags(int flags)
{
    Xila_memory_flag_type Xila_flags = Xila_memory_flag_anonymous | Xila_memory_flag_private;

    if (flags & MMAP_MAP_FIXED)
        Xila_flags |= Xila_memory_flag_fixed;

    if (flags & MMAP_MAP_32BIT)
        Xila_flags |= Xila_memory_flag_address_32_bits;

    return Xila_flags;
}

void *os_mmap(void *hint, size_t size, int prot, int flags, os_file_handle file)
{
    Xila_memory_protection_type Xila_protection = To_xila_memory_protection(prot);

    Xila_memory_flag_type Xila_flags = To_xila_memory_flags(flags);

    return Xila_memory_allocate_custom(hint, size, 8, Xila_protection, Xila_flags);
}

void os_munmap(void *addr, size_t size)
{
    Xila_memory_deallocate_custom(addr, size);
}

int os_mprotect(void *addr, size_t size, int prot)
{
    Xila_memory_protection_type Xila_protection = To_xila_memory_protection(prot);

    return !Xila_memory_protect(addr, size, Xila_protection);
}

int os_getpagesize()
{
    return Xila_memory_get_page_size();
}

/* Doesn't guarantee that protection flags will be preserved.
   os_mprotect() must be called after remapping. */
void *os_mremap(void *old_addr, size_t old_size, size_t new_size)
{
    return os_mremap_slow(old_addr, old_size, new_size);
}

#if (WASM_MEM_DUAL_BUS_MIRROR != 0)
void *
os_get_dbus_mirror(void *ibus);
#endif

/**
 * Flush cpu data cache, in some CPUs, after applying relocation to the
 * AOT code, the code may haven't been written back to the cpu data cache,
 * which may cause unexpected behaviour when executing the AOT code.
 * Implement this function if required, or just leave it empty.
 */
void os_dcache_flush(void)
{
    Xila_flush_data_cache();
}

/**
 * Flush instruction cache.
 */
void os_icache_flush(void *start, size_t len)
{
    Xila_flush_instruction_cache(start, len);
}
