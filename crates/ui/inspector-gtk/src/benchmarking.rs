pub(super) fn widget() -> gtk::Widget {
    let section = shrimply_inspector_core::benchmarking::section();
    let [control] = section.controls.as_slice() else {
        panic!("performance inspector must have one control");
    };
    assert_eq!(
        control.kind,
        shrimply_inspector_core::ControlKind::Performance
    );
    assert!(!control.editable);
    assert!(control.visible);
    assert!(control.sensitive);
    shrimply_gtk_components::ui::live_performance()
}
