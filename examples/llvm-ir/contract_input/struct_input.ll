; ModuleID = 'MultInputExample'
source_filename = "examples/source/contract_input/struct_input.ola"

@heap_address = internal global i64 -4294967353

declare void @builtin_assert(i64)

declare void @builtin_range_check(i64)

define void @mempcy(ptr %0, ptr %1, i64 %2) {
entry:
  %index_alloca = alloca i64, align 8
  %len_alloca = alloca i64, align 8
  %dest_ptr_alloca = alloca ptr, align 8
  %src_ptr_alloca = alloca ptr, align 8
  store ptr %0, ptr %src_ptr_alloca, align 8
  %src_ptr = load ptr, ptr %src_ptr_alloca, align 8
  store ptr %1, ptr %dest_ptr_alloca, align 8
  %dest_ptr = load ptr, ptr %dest_ptr_alloca, align 8
  store i64 %2, ptr %len_alloca, align 4
  %len = load i64, ptr %len_alloca, align 4
  store i64 0, ptr %index_alloca, align 4
  br label %cond

cond:                                             ; preds = %body, %entry
  %index_value = load i64, ptr %index_alloca, align 4
  %loop_cond = icmp ult i64 %index_value, %len
  br i1 %loop_cond, label %body, label %done

body:                                             ; preds = %cond
  %src_index_access = getelementptr i64, ptr %src_ptr, i64 %index_value
  %3 = load i64, ptr %src_index_access, align 4
  %dest_index_access = getelementptr i64, ptr %dest_ptr, i64 %index_value
  store i64 %3, ptr %dest_index_access, align 4
  %next_index = add i64 %index_value, 1
  store i64 %next_index, ptr %index_alloca, align 4
  br label %cond

done:                                             ; preds = %cond
  ret void
}

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

declare void @prophet_printf(i64, i64)

define void @foo(ptr %0) {
entry:
  %t = alloca ptr, align 8
  store ptr %0, ptr %t, align 8
  %1 = load ptr, ptr %t, align 8
  %struct_member = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 0
  %2 = load ptr, ptr %struct_member, align 8
  %address_start = ptrtoint ptr %2 to i64
  call void @prophet_printf(i64 %address_start, i64 2)
  %struct_member1 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 1
  %3 = load i64, ptr %struct_member1, align 4
  call void @prophet_printf(i64 %3, i64 3)
  %struct_member2 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 2
  %4 = load i64, ptr %struct_member2, align 4
  call void @prophet_printf(i64 %4, i64 3)
  %struct_member3 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 3
  %5 = load i64, ptr %struct_member3, align 4
  call void @prophet_printf(i64 %5, i64 3)
  %struct_member4 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 4
  %6 = load ptr, ptr %struct_member4, align 8
  %fields_start = ptrtoint ptr %6 to i64
  call void @prophet_printf(i64 %fields_start, i64 1)
  %struct_member5 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 5
  %7 = load ptr, ptr %struct_member5, align 8
  %fields_start6 = ptrtoint ptr %7 to i64
  call void @prophet_printf(i64 %fields_start6, i64 1)
  %struct_member7 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 6
  %8 = load ptr, ptr %struct_member7, align 8
  %fields_start8 = ptrtoint ptr %8 to i64
  call void @prophet_printf(i64 %fields_start8, i64 1)
  %struct_member9 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %1, i64 7
  %9 = load ptr, ptr %struct_member9, align 8
  %hash_start = ptrtoint ptr %9 to i64
  call void @prophet_printf(i64 %hash_start, i64 2)
  ret void
}

define void @function_dispatch(i64 %0, i64 %1, ptr %2) {
entry:
  %input_alloca = alloca ptr, align 8
  store ptr %2, ptr %input_alloca, align 8
  %input = load ptr, ptr %input_alloca, align 8
  switch i64 %0, label %missing_function [
    i64 3469705383, label %func_0_dispatch
  ]

missing_function:                                 ; preds = %entry
  unreachable

func_0_dispatch:                                  ; preds = %entry
  %input_start = ptrtoint ptr %input to i64
  %3 = inttoptr i64 %input_start to ptr
  %struct_offset = add i64 %input_start, 4
  %4 = inttoptr i64 %struct_offset to ptr
  %decode_value = load i64, ptr %4, align 4
  %struct_offset1 = add i64 %struct_offset, 1
  %5 = inttoptr i64 %struct_offset1 to ptr
  %decode_value2 = load i64, ptr %5, align 4
  %struct_offset3 = add i64 %struct_offset1, 1
  %6 = inttoptr i64 %struct_offset3 to ptr
  %decode_value4 = load i64, ptr %6, align 4
  %struct_offset5 = add i64 %struct_offset3, 1
  %7 = inttoptr i64 %struct_offset5 to ptr
  %length = load i64, ptr %7, align 4
  %8 = add i64 %length, 1
  %struct_offset6 = add i64 %struct_offset5, %8
  %9 = inttoptr i64 %struct_offset6 to ptr
  %length7 = load i64, ptr %9, align 4
  %10 = add i64 %length7, 1
  %struct_offset8 = add i64 %struct_offset6, %10
  %11 = inttoptr i64 %struct_offset8 to ptr
  %length9 = load i64, ptr %11, align 4
  %12 = add i64 %length9, 1
  %struct_offset10 = add i64 %struct_offset8, %12
  %13 = inttoptr i64 %struct_offset10 to ptr
  %struct_offset11 = add i64 %struct_offset10, 4
  %struct_decode_size = sub i64 %struct_offset11, %input_start
  %14 = call i64 @vector_new(i64 14)
  %heap_start = sub i64 %14, 14
  %heap_to_ptr = inttoptr i64 %heap_start to ptr
  %struct_member = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 0
  store ptr %3, ptr %struct_member, align 8
  %struct_member12 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 1
  store i64 %decode_value, ptr %struct_member12, align 4
  %struct_member13 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 2
  store i64 %decode_value2, ptr %struct_member13, align 4
  %struct_member14 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 3
  store i64 %decode_value4, ptr %struct_member14, align 4
  %struct_member15 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 4
  store ptr %7, ptr %struct_member15, align 8
  %struct_member16 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 5
  store ptr %9, ptr %struct_member16, align 8
  %struct_member17 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 6
  store ptr %11, ptr %struct_member17, align 8
  %struct_member18 = getelementptr { ptr, i64, i64, i64, ptr, ptr, ptr, ptr }, ptr %heap_to_ptr, i64 7
  store ptr %13, ptr %struct_member18, align 8
  call void @foo(ptr %heap_to_ptr)
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