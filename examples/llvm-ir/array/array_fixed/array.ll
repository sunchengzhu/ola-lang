; ModuleID = 'FixedArrayExample'
source_filename = "examples/source/array/array_fixed/array.ola"

@heap_address = internal global i64 -4294967353

declare void @builtin_assert(i64)

declare void @builtin_range_check(i64)

declare i64 @prophet_u32_sqrt(i64)

declare i64 @prophet_u32_div(i64, i64)

declare i64 @prophet_u32_mod(i64, i64)

declare ptr @prophet_u32_array_sort(ptr, i64)

declare i64 @vector_new(i64)

declare void @get_context_data(i64, i64)

declare void @get_call_data(i64, i64)

declare void @set_tape_data(i64, i64)

declare [4 x i64] @get_storage([4 x i64])

declare void @set_storage([4 x i64], [4 x i64])

declare [4 x i64] @poseidon_hash([8 x i64])

define void @main() {
entry:
  %array_literal6 = alloca [3 x i64], align 8
  %array_literal2 = alloca [3 x i64], align 8
  %array_literal = alloca [3 x i64], align 8
  %elemptr0 = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 0
  store i64 1, ptr %elemptr0, align 4
  %elemptr1 = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 1
  store i64 2, ptr %elemptr1, align 4
  %elemptr2 = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 2
  store i64 3, ptr %elemptr2, align 4
  call void @builtin_range_check(i64 1)
  %index_access = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 1
  call void @builtin_range_check(i64 1)
  %index_access1 = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 1
  %0 = load i64, ptr %index_access1, align 4
  %1 = add i64 %0, 1
  call void @builtin_range_check(i64 %1)
  store i64 %1, ptr %index_access, align 4
  %elemptr03 = getelementptr [3 x i64], ptr %array_literal2, i64 0, i64 0
  store i64 1, ptr %elemptr03, align 4
  %elemptr14 = getelementptr [3 x i64], ptr %array_literal2, i64 0, i64 1
  store i64 2, ptr %elemptr14, align 4
  %elemptr25 = getelementptr [3 x i64], ptr %array_literal2, i64 0, i64 2
  store i64 3, ptr %elemptr25, align 4
  %elemptr07 = getelementptr [3 x i64], ptr %array_literal6, i64 0, i64 0
  store i64 0, ptr %elemptr07, align 4
  %elemptr18 = getelementptr [3 x i64], ptr %array_literal6, i64 0, i64 1
  store i64 0, ptr %elemptr18, align 4
  %elemptr29 = getelementptr [3 x i64], ptr %array_literal6, i64 0, i64 2
  store i64 0, ptr %elemptr29, align 4
  call void @builtin_range_check(i64 0)
  %index_access10 = getelementptr [3 x i64], ptr %array_literal6, i64 0, i64 2
  store i64 99, ptr %index_access10, align 4
  %2 = call ptr @array_call(ptr %array_literal6)
  ret void
}

define ptr @array_call(ptr %0) {
entry:
  call void @builtin_range_check(i64 0)
  %index_access = getelementptr [3 x i64], ptr %0, i64 0, i64 2
  store i64 100, ptr %index_access, align 4
  ret ptr %0
}

define void @function_dispatch(i64 %0, i64 %1, ptr %2) {
entry:
  %array_literal = alloca [3 x i64], align 8
  switch i64 %0, label %missing_function [
    i64 3501063903, label %func_0_dispatch
    i64 984717406, label %func_1_dispatch
  ]

missing_function:                                 ; preds = %entry
  unreachable

func_0_dispatch:                                  ; preds = %entry
  call void @main()
  ret void

func_1_dispatch:                                  ; preds = %entry
  %elemptr0 = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 0
  %start = getelementptr i64, ptr %2, i64 0
  %value = load i64, ptr %start, align 4
  store i64 %value, ptr %elemptr0, align 4
  %elemptr1 = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 1
  %start1 = getelementptr i64, ptr %2, i64 1
  %value2 = load i64, ptr %start1, align 4
  store i64 %value2, ptr %elemptr1, align 4
  %elemptr2 = getelementptr [3 x i64], ptr %array_literal, i64 0, i64 2
  %start3 = getelementptr i64, ptr %2, i64 2
  %value4 = load i64, ptr %start3, align 4
  store i64 %value4, ptr %elemptr2, align 4
  %3 = icmp ule i64 3, %1
  br i1 %3, label %inbounds, label %out_of_bounds

inbounds:                                         ; preds = %func_1_dispatch
  %4 = icmp ult i64 3, %1
  br i1 %4, label %not_all_bytes_read, label %buffer_read

out_of_bounds:                                    ; preds = %func_1_dispatch
  unreachable

not_all_bytes_read:                               ; preds = %inbounds
  unreachable

buffer_read:                                      ; preds = %inbounds
  %5 = call ptr @array_call(ptr %array_literal)
  %6 = call i64 @vector_new(i64 3)
  %heap_start = sub i64 %6, 3
  %heap_to_ptr = inttoptr i64 %heap_start to ptr
  %elemptr05 = getelementptr [3 x i64], ptr %5, i64 0, i64 0
  %7 = load i64, ptr %elemptr05, align 4
  %start6 = getelementptr i64, ptr %heap_to_ptr, i64 0
  store i64 %7, ptr %start6, align 4
  %elemptr17 = getelementptr [3 x i64], ptr %5, i64 0, i64 1
  %8 = load i64, ptr %elemptr17, align 4
  %start8 = getelementptr i64, ptr %heap_to_ptr, i64 1
  store i64 %8, ptr %start8, align 4
  %elemptr29 = getelementptr [3 x i64], ptr %5, i64 0, i64 2
  %9 = load i64, ptr %elemptr29, align 4
  %start10 = getelementptr i64, ptr %heap_to_ptr, i64 2
  store i64 %9, ptr %start10, align 4
  call void @set_tape_data(i64 %heap_start, i64 3)
  ret void
}

define void @call() {
entry:
  %0 = call i64 @vector_new(i64 1)
  %heap_start = sub i64 %0, 1
  %heap_to_ptr = inttoptr i64 %heap_start to ptr
  call void @get_call_data(i64 %heap_start, i64 0)
  %function_selector = load i64, ptr %heap_to_ptr, align 4
  %1 = call i64 @vector_new(i64 1)
  %heap_start1 = sub i64 %1, 1
  %heap_to_ptr2 = inttoptr i64 %heap_start1 to ptr
  call void @get_call_data(i64 %heap_start1, i64 1)
  %input_length = load i64, ptr %heap_to_ptr2, align 4
  %2 = call i64 @vector_new(i64 %input_length)
  %heap_start3 = sub i64 %2, %input_length
  %heap_to_ptr4 = inttoptr i64 %heap_start3 to ptr
  call void @get_call_data(i64 %heap_start3, i64 2)
  call void @function_dispatch(i64 %function_selector, i64 %input_length, ptr %heap_to_ptr4)
  unreachable
}
