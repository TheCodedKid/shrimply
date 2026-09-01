import dev.shrimply.components

InspectorPairGraphProperty {
    id: root
    onGraphReset: function(component, value) { loader.load(component, value) }
    DemoGraphLoader { id: loader; target: root }
}
