comptime {
    @export(entry, .{ .name = "entry", .linkage = .Strong });
}

var counter: f32 = 0;

pub fn entry() callconv(.C) f32 {
    counter += 1.0;
    return counter;
}
