#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <windows.h>

#ifdef __cplusplus
extern "C" {
#endif

// 1. __std_max_8i
// Returns the maximum 64-bit signed integer in the range [_First, _Last).
int64_t __stdcall __std_max_8i(const void* _First, const void* _Last) {
    const int64_t* cur = (const int64_t*)_First;
    const int64_t* end = (const int64_t*)_Last;
    if (cur >= end) return 0;
    int64_t m = *cur++;
    while (cur < end) {
        if (*cur > m) {
            m = *cur;
        }
        cur++;
    }
    return m;
}

// 2. __std_min_8i
// Returns the minimum 64-bit signed integer in the range [_First, _Last).
int64_t __stdcall __std_min_8i(const void* _First, const void* _Last) {
    const int64_t* cur = (const int64_t*)_First;
    const int64_t* end = (const int64_t*)_Last;
    if (cur >= end) return 0;
    int64_t m = *cur++;
    while (cur < end) {
        if (*cur < m) {
            m = *cur;
        }
        cur++;
    }
    return m;
}

// 3. __std_min_4u
// Returns the minimum 32-bit unsigned integer in the range [_First, _Last).
uint32_t __stdcall __std_min_4u(const void* _First, const void* _Last) {
    const uint32_t* cur = (const uint32_t*)_First;
    const uint32_t* end = (const uint32_t*)_Last;
    if (cur >= end) return 0;
    uint32_t m = *cur++;
    while (cur < end) {
        if (*cur < m) {
            m = *cur;
        }
        cur++;
    }
    return m;
}

// 4. __std_max_element_d and __std_max_element_d_
// Returns pointer to the maximum double in the range [_First, _Last).
const void* __stdcall __std_max_element_d(const void* _First, const void* _Last) {
    const double* cur = (const double*)_First;
    const double* end = (const double*)_Last;
    if (cur >= end) return _First;
    const double* max_ptr = cur++;
    while (cur < end) {
        if (*cur > *max_ptr) {
            max_ptr = cur;
        }
        cur++;
    }
    return (const void*)max_ptr;
}

const void* __stdcall __std_max_element_d_(const void* _First, const void* _Last) {
    return __std_max_element_d(_First, _Last);
}

// 5. __std_mismatch_1
// Compares 1-byte elements, returns index of first mismatch or _Count if equal.
size_t __stdcall __std_mismatch_1(const void* _First1, const void* _First2, size_t _Count) {
    const uint8_t* p1 = (const uint8_t*)_First1;
    const uint8_t* p2 = (const uint8_t*)_First2;
    for (size_t i = 0; i < _Count; ++i) {
        if (p1[i] != p2[i]) {
            return i;
        }
    }
    return _Count;
}

// 6. __std_mismatch_2
// Compares 2-byte elements, returns index of first mismatch or _Count if equal.
size_t __stdcall __std_mismatch_2(const void* _First1, const void* _First2, size_t _Count) {
    const uint16_t* p1 = (const uint16_t*)_First1;
    const uint16_t* p2 = (const uint16_t*)_First2;
    for (size_t i = 0; i < _Count; ++i) {
        if (p1[i] != p2[i]) {
            return i;
        }
    }
    return _Count;
}

// 7. __std_search_1
// Searches for sequence _First2 (count _Count2) in [_First1, _Last1).
// Returns pointer to first occurrence or _Last1 if not found.
const void* __stdcall __std_search_1(const void* _First1, const void* _Last1, const void* _First2, size_t _Count2) {
    const uint8_t* h = (const uint8_t*)_First1;
    const uint8_t* h_end = (const uint8_t*)_Last1;
    const uint8_t* n = (const uint8_t*)_First2;
    
    if (_Count2 == 0) return _First1;
    if ((size_t)(h_end - h) < _Count2) return _Last1;
    
    const uint8_t* limit = h_end - _Count2;
    for (const uint8_t* p = h; p <= limit; ++p) {
        if (memcmp(p, n, _Count2) == 0) {
            return (const void*)p;
        }
    }
    return _Last1;
}

// 8. __std_find_first_of_trivial_pos_1
// Searches for the first element in _Haystack matching ANY element in _Needles.
// Returns 0-based offset or (size_t)-1 if not found.
size_t __stdcall __std_find_first_of_trivial_pos_1(const void* _Haystack, size_t _Haystack_count, const void* _Needles, size_t _Needles_count) {
    const uint8_t* h = (const uint8_t*)_Haystack;
    const uint8_t* n = (const uint8_t*)_Needles;
    
    if (_Haystack_count == 0 || _Needles_count == 0) {
        return (size_t)-1;
    }
    
    // Quick bitmap lookup table for 1-byte characters
    uint8_t table[256];
    memset(table, 0, sizeof(table));
    for (size_t j = 0; j < _Needles_count; ++j) {
        table[n[j]] = 1;
    }
    
    for (size_t i = 0; i < _Haystack_count; ++i) {
        if (table[h[i]]) {
            return i;
        }
    }
    
    return (size_t)-1;
}

// 9. _Thrd_sleep_for
// Suspend current thread for `ms` milliseconds.
void __cdecl _Thrd_sleep_for(unsigned long ms) {
    Sleep(ms);
}

void __stdcall _Thrd_sleep_for_stdcall(unsigned long ms) {
    Sleep(ms);
}

#ifdef __cplusplus
}
#endif
