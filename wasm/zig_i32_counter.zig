comptime {
    @export(entry, .{ .name = "entry", .linkage = .Strong });
}

var counter: i32 = 0;

pub fn entry() callconv(.C) i32 {
    counter += 1;
    return counter;
}
