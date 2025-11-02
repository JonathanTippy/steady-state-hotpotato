use steady_state::*;
use arg::MainArg;

const PLAYER_NAME:&str = "player";
const NUM_OF_PLAYERS:usize = 4;

const NUMBERS: [i32; 5] = [1, 2, 3, 4, 5];

const NAMES: [&str; 5] = ["player1", "player2", "player3", "player4", "player5"];


mod arg;




/// Actor module organization demonstrates scalable code structure.
/// This pattern enables clean separation of concerns while maintaining
/// visibility and reusability across different deployment configurations.
pub(crate) mod actor {

    pub(crate) mod player;
}


use crate::actor::player::Potato;

/// Application entry point demonstrating production-ready initialization patterns.
/// This includes command-line processing, logging setup, graph construction,
/// and lifecycle management with proper error handling and resource cleanup.
fn main() -> Result<(), Box<dyn Error>> {

    let cli_args = MainArg::parse();
    init_logging(LogLevel::Info)?;
    let mut graph = GraphBuilder::default()
        // The default and minimum telemetry frame rate is 40ms. It works well for most cases.
        //.with_telemtry_production_rate_ms(200) //You can slow it down with this  //#!#//
        .build(cli_args);

    build_graph(&mut graph);

    // Synchronous startup ensures all actors are ready before proceeding.
    // This prevents race conditions during initialization and provides
    // predictable system behavior from the start.
    graph.start();
    // Blocking wait with timeout prevents infinite hangs while allowing
    // graceful shutdown completion. The timeout you set should be larger than
    // the expected cleanup duration for all actors to avoid premature termination.
    graph.block_until_stopped(Duration::from_secs(4))
}

fn build_graph(graph: &mut Graph) {

    // Channel builder configuration applies consistent monitoring across all channels.
    // This provides uniform observability and alerting behavior without requiring
    // individual channel configuration or runtime performance analysis.
    let channel_builder = graph.channel_builder()
        // Threshold-based alerting enables proactive monitoring of system health.
        // Red alerts indicate critical congestion requiring immediate attention,
        // while orange alerts provide early warning of developing bottlenecks.
        .with_filled_trigger(Trigger::AvgAbove(Filled::p90()), AlertColor::Red) //#!#//
        .with_filled_trigger(Trigger::AvgAbove(Filled::p60()), AlertColor::Orange)
        // Percentile monitoring provides statistical insight into channel utilization.
        .with_filled_percentile(Percentile::p80());

    let mut channels:Vec<(LazySteadyTx<Potato>, LazySteadyRx<Potato>)> = vec!();
    for _ in 0..NUM_OF_PLAYERS {
        channels.push(channel_builder.build());
    }

    let actor_builder = graph.actor_builder()
        // Load distribution metrics enable capacity planning and bottleneck identification.
        // This shows which actors consume the most resources relative to graph capacity.
        .with_load_avg()//#!#//
        // CPU utilization tracking provides absolute performance measurement.
        // Values are normalized to 1024 units per core for consistent cross-platform metrics.
        .with_mcpu_avg();//#!#//


    for i in 0..NUM_OF_PLAYERS {


        let tx = channels[i].0.clone();
        let rx = channels[(i+1)%NUM_OF_PLAYERS].1.clone();




        actor_builder.with_name(NAMES[i]).build(move |actor| actor::player::run(actor, rx.clone(), tx.clone(), i==0)
                                    , SoloAct);



    }

}
