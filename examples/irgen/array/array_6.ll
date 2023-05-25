; ModuleID = 'ArraySortExample'
source_filename = "examples/source/array/array_6.ola"

declare void @builtin_assert(i64, i64)

declare void @builtin_range_check(i64)

declare i64 @prophet_u32_sqrt(i64)

declare i64 @prophet_u32_div(i64, i64)

declare i64 @prophet_u32_mod(i64, i64)

declare ptr @prophet_u32_array_sort(ptr, i64)

declare ptr @prophet_malloc(i64)

define void @main() {
entry:
  %sorted = alloca ptr, align 8
  %0 = alloca ptr, align 8
  %source2 = alloca ptr, align 8
  %source = alloca ptr, align 8
  %array_literal = alloca [10 x i64], align 8
  %elemptr0 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 0
  store i64 3, ptr %elemptr0, align 4
  %elemptr1 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 1
  store i64 4, ptr %elemptr1, align 4
  %elemptr2 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 2
  store i64 5, ptr %elemptr2, align 4
  %elemptr3 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 3
  store i64 1, ptr %elemptr3, align 4
  %elemptr4 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 4
  store i64 7, ptr %elemptr4, align 4
  %elemptr5 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 5
  store i64 9, ptr %elemptr5, align 4
  %elemptr6 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 6
  store i64 0, ptr %elemptr6, align 4
  %elemptr7 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 7
  store i64 2, ptr %elemptr7, align 4
  %elemptr8 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 8
  store i64 8, ptr %elemptr8, align 4
  %elemptr9 = getelementptr [10 x i64], ptr %array_literal, i64 0, i64 9
  store i64 6, ptr %elemptr9, align 4
  store ptr %array_literal, ptr %source, align 8
  %1 = load ptr, ptr %source, align 8
  store ptr %1, ptr %source2, align 8
  %2 = load ptr, ptr %source2, align 8
  %3 = call ptr @array_sort_test(ptr %2)
  store ptr %3, ptr %0, align 8
  store ptr %0, ptr %sorted, align 8
  ret void
}

define ptr @array_sort_test(ptr %0) {
entry:
  %array_sorted = alloca ptr, align 8
  %source = alloca ptr, align 8
  store ptr %0, ptr %source, align 8
  %1 = load ptr, ptr %source, align 8
  %2 = call ptr @prophet_u32_array_sort(ptr %1, i64 10)
  store ptr %2, ptr %array_sorted, align 8
  %3 = load ptr, ptr %array_sorted, align 8
  ret ptr %3
}