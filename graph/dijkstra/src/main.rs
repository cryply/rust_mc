// use std::ffi::IntoStringError;

use petgraph::{Graph,};
use petgraph::algo::dijkstra;
use petgraph::prelude::*;


fn main() {
    let mut graph: Graph<&str, i32, petgraph::Undirected> = Graph::new_undirected();

    let munich = graph.add_node("Munich");
    let innsbruck = graph.add_node("Innsbruck");
    let salzburg = graph.add_node("Salzburg");
    let bolzano = graph.add_node("Bolzano");
    let trento = graph.add_node("Trento");
    let vienna = graph.add_node("Vienna");
    let bratislava = graph.add_node("Bratislava");
    let budapest = graph.add_node("Budapest");

    graph.add_edge(munich, innsbruck, 142);
    graph.add_edge(munich, salzburg,    150);
    graph.add_edge(innsbruck, salzburg,    189);
    graph.add_edge(innsbruck, bolzano,    121);
    graph.add_edge(bolzano, trento,    58);

    graph.add_edge(salzburg, vienna,    296);

    graph.add_edge(vienna,  bratislava,    80);
    graph.add_edge(vienna, budapest,    290);
    graph.add_edge(bratislava, budapest,    200);

    let path = dijkstra(&graph, munich, Some(trento), |w| *w.weight(), );
    
    // match path {
    //     Some((cost, path)) => {
    //         println!("Total distance: {}: {:?}", cost, path);
    //     } 
    //     None => println!("No path")
    // }
    println!("{:?}", path);

    path.iter().for_each(|n| {
        let city  = graph.node_weight(*n.0).unwrap();
        println!("{:?} : {}", city, n.1);
    })

}
