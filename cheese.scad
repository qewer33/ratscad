// ratSCAD cheese demo
$fn = 50;

// Cheese!
color("yellow") mirror([1, 0, 0]) difference() {
    // Base triangle
    linear_extrude(13) polygon([[0, 0], [10, 25], [20, 0]]);

    // Cut spheres
    union() {
        translate([14,3, 15]) sphere(3);
        translate([10, 6, 14]) sphere(3);
        translate([5, 16, 14]) sphere(5);
        translate([-1, 6, 0]) sphere(5);
        translate([3, 17, 3]) sphere(4);
    }
}
