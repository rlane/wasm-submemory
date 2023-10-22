comptime {
    @export(entry, .{ .name = "entry", .linkage = .Strong });
}

comptime {
    @export(inc, .{ .name = "inc", .linkage = .Strong });
}

var data: i32 = 42;

pub fn entry() callconv(.C) i32 {
    return data;
}

pub fn inc() callconv(.C) void {
    data += 1;
}
