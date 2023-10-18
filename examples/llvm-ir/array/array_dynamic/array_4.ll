; ModuleID = 'DynamicArrayExample'
source_filename = "examples/source/array/array_dynamic/array_4.ola"

@heap_address = internal global i64 -4294967353

declare void @builtin_assert(i64)

declare void @builtin_range_check(i64)

declare i64 @prophet_u32_sqrt(i64)

declare i64 @prophet_u32_div(i64, i64)

declare i64 @prophet_u32_mod(i64, i64)

declare ptr @prophet_u32_array_sort(ptr, i64)

declare i64 @vector_new(i64)

declare void @get_context_data(i64, i64)

declare void @get_tape_data(i64, i64)

declare void @set_tape_data(i64, i64)

declare void @get_storage(ptr, ptr)

declare void @set_storage(ptr, ptr)

declare void @poseidon_hash(ptr, ptr, i64)

declare void @contract_call(ptr, i64)

define void @test() {
entry:
  %index_alloca = alloca i64, align 8
  %0 = call i64 @vector_new(i64 6)
  %heap_start = sub i64 %0, 6
  %heap_to_ptr = inttoptr i64 %heap_start to ptr
  store i64 5, ptr %heap_to_ptr, align 4
  %1 = ptrtoint ptr %heap_to_ptr to i64
  %2 = add i64 %1, 1
  %vector_data = inttoptr i64 %2 to ptr
  store i64 0, ptr %index_alloca, align 4
  br label %cond

cond:                                             ; preds = %body, %entry
  %index_value = load i64, ptr %index_alloca, align 4
  %loop_cond = icmp ult i64 %index_value, 5
  br i1 %loop_cond, label %body, label %done

body:                                             ; preds = %cond
  %index_access = getelementptr i64, ptr %vector_data, i64 %index_value
  store i64 0, ptr %index_access, align 4
  %next_index = add i64 %index_value, 1
  store i64 %next_index, ptr %index_alloca, align 4
  br label %cond

done:                                             ; preds = %cond
  %length = load i64, ptr %heap_to_ptr, align 4
  %3 = sub i64 %length, 1
  %4 = sub i64 %3, 0
  call void @builtin_range_check(i64 %4)
  %5 = ptrtoint ptr %heap_to_ptr to i64
  %6 = add i64 %5, 1
  %vector_data1 = inttoptr i64 %6 to ptr
  %index_access2 = getelementptr i64, ptr %vector_data1, i64 0
  store i64 1, ptr %index_access2, align 4
  %length3 = load i64, ptr %heap_to_ptr, align 4
  %7 = sub i64 %length3, 1
  %8 = sub i64 %7, 0
  call void @builtin_range_check(i64 %8)
  %9 = ptrtoint ptr %heap_to_ptr to i64
  %10 = add i64 %9, 1
  %vector_data4 = inttoptr i64 %10 to ptr
  %index_access5 = getelementptr i64, ptr %vector_data4, i64 0
  %11 = load i64, ptr %index_access5, align 4
  %12 = icmp eq i64 %11, 1
  %13 = zext i1 %12 to i64
  call void @builtin_assert(i64 %13)
  %14 = call ptr @array_call(i64 5)
  %length6 = load i64, ptr %14, align 4
  %15 = sub i64 %length6, 1
  %16 = sub i64 %15, 0
  call void @builtin_range_check(i64 %16)
  %17 = ptrtoint ptr %14 to i64
  %18 = add i64 %17, 1
  %vector_data7 = inttoptr i64 %18 to ptr
  %index_access8 = getelementptr i64, ptr %vector_data7, i64 0
  %19 = load i64, ptr %index_access8, align 4
  %20 = icmp eq i64 %19, 0
  %21 = zext i1 %20 to i64
  call void @builtin_assert(i64 %21)
  ret void
}

define ptr @array_call(i64 %0) {
entry:
  %index_alloca = alloca i64, align 8
  %length = alloca i64, align 8
  store i64 %0, ptr %length, align 4
  %1 = load i64, ptr %length, align 4
  %size = mul i64 %1, 1
  %size_add_one = add i64 %size, 1
  %2 = call i64 @vector_new(i64 %size_add_one)
  %heap_start = sub i64 %2, %size_add_one
  %heap_to_ptr = inttoptr i64 %heap_start to ptr
  store i64 %size, ptr %heap_to_ptr, align 4
  %3 = ptrtoint ptr %heap_to_ptr to i64
  %4 = add i64 %3, 1
  %vector_data = inttoptr i64 %4 to ptr
  store i64 0, ptr %index_alloca, align 4
  br label %cond

cond:                                             ; preds = %body, %entry
  %index_value = load i64, ptr %index_alloca, align 4
  %loop_cond = icmp ult i64 %index_value, %1
  br i1 %loop_cond, label %body, label %done

body:                                             ; preds = %cond
  %index_access = getelementptr i64, ptr %vector_data, i64 %index_value
  store i64 0, ptr %index_access, align 4
  %next_index = add i64 %index_value, 1
  store i64 %next_index, ptr %index_alloca, align 4
  br label %cond

done:                                             ; preds = %cond
  ret ptr %heap_to_ptr
}

define void @function_dispatch(i64 %0, i64 %1, ptr %2) {
entry:
  switch i64 %0, label %missing_function [
    i64 1845340408, label %func_0_dispatch
    i64 991959678, label %func_1_dispatch
  ]

missing_function:                                 ; preds = %entry
  unreachable

func_0_dispatch:                                  ; preds = %entry
  call void @test()
  ret void

func_1_dispatch:                                  ; preds = %entry
  %3 = icmp ule i64 1, %1
  br i1 %3, label %inbounds, label %out_of_bounds

inbounds:                                         ; preds = %func_1_dispatch
  %start = getelementptr i64, ptr %2, i64 0
  %value = load i64, ptr %start, align 4
  %4 = icmp ult i64 1, %1
  br i1 %4, label %not_all_bytes_read, label %buffer_read

out_of_bounds:                                    ; preds = %func_1_dispatch
  unreachable

not_all_bytes_read:                               ; preds = %inbounds
  unreachable

buffer_read:                                      ; preds = %inbounds
  %5 = call ptr @array_call(i64 %value)
  %length = load i64, ptr %5, align 4
  %6 = mul i64 %length, 1
  %7 = add i64 %6, 1
  %heap_size = add i64 %7, 1
  %8 = call i64 @vector_new(i64 %heap_size)
  %heap_start = sub i64 %8, %heap_size
  %heap_to_ptr = inttoptr i64 %heap_start to ptr
  %length1 = load i64, ptr %5, align 4
  %start2 = getelementptr i64, ptr %heap_to_ptr, i64 0
  store i64 %length1, ptr %start2, align 4
  %index_ptr = alloca i64, align 8
  store i64 0, ptr %index_ptr, align 4
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %buffer_read
  %index = load i64, ptr %index_ptr, align 4
  %element = getelementptr ptr, ptr %5, i64 %index
  %elem = load i64, ptr %element, align 4
  %start3 = getelementptr i64, ptr %heap_to_ptr, i64 1
  store i64 %elem, ptr %start3, align 4
  %next_index = add i64 %index, 1
  store i64 %next_index, ptr %index_ptr, align 4
  %index_cond = icmp ult i64 %next_index, %length1
  br i1 %index_cond, label %loop_body, label %loop_end

loop_end:                                         ; preds = %loop_body
  %9 = add i64 %length1, 1
  %10 = add i64 0, %9
  %start4 = getelementptr i64, ptr %heap_to_ptr, i64 %10
  store i64 %7, ptr %start4, align 4
  call void @set_tape_data(i64 %heap_start, i64 %heap_size)
  ret void
}

define void @main() {
entry:
  %0 = call i64 @vector_new(i64 13)
  %heap_start = sub i64 %0, 13
  %heap_to_ptr = inttoptr i64 %heap_start to ptr
  call void @get_tape_data(i64 %heap_start, i64 13)
  %function_selector = load i64, ptr %heap_to_ptr, align 4
  %1 = call i64 @vector_new(i64 14)
  %heap_start1 = sub i64 %1, 14
  %heap_to_ptr2 = inttoptr i64 %heap_start1 to ptr
  call void @get_tape_data(i64 %heap_start1, i64 14)
  %input_length = load i64, ptr %heap_to_ptr2, align 4
  %2 = add i64 %input_length, 14
  %3 = call i64 @vector_new(i64 %2)
  %heap_start3 = sub i64 %3, %2
  %heap_to_ptr4 = inttoptr i64 %heap_start3 to ptr
  call void @get_tape_data(i64 %heap_start3, i64 %2)
  call void @function_dispatch(i64 %function_selector, i64 %input_length, ptr %heap_to_ptr4)
  ret void
}
