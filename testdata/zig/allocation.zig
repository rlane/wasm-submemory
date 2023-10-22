comptime {
    @export(entry, .{ .name = "entry", .linkage = .Strong });
}

const std = @import("std");

pub fn entry() callconv(.C) i32 {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const allocator = gpa.allocator();
    defer _ = gpa.deinit();

    const bytes = allocator.alloc(u8, 100) catch unreachable;
    defer allocator.free(bytes);
    return 42;
}
