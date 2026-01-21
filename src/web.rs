use crate::state::SharedState;
use axum::{
    extract::{State, Json},
    response::Html,
    routing::get,
    Router,
};
use serde_json::Value;

pub async fn start_server(state: SharedState, port: u16) {
    let app = Router::new()
        .route("/", get(index))
        .route("/live", get(get_live))
        .route("/races", get(get_races))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Web server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Lynx vMix Bridge</title>
        <style>
            body { font-family: monospace; background: #000; color: #0f0; padding: 20px; }
            h1 { border-bottom: 1px solid #333; }
            #time { font-size: 4em; font-weight: bold; }
            table { width: 100%; border-collapse: collapse; margin-top: 20px; }
            th, td { border: 1px solid #333; padding: 5px; text-align: left; }
            th { background: #111; }
        </style>
        <script>
            async function poll() {
                try {
                    let res = await fetch('/live');
                    let json = await res.json();
                    let data = json[0];
                    
                    document.getElementById('time').innerText = data.time || "--:--.--";
                    
                    let tbody = document.getElementById('results');
                    tbody.innerHTML = '';
                    data.results.forEach(r => {
                        let row = `<tr>
                            <td>${r.place}</td>
                            <td>${r.lane}</td>
                            <td>${r.id}</td>
                            <td>${r.name}</td>
                            <td>${r.affiliation}</td>
                            <td>${r.time}</td>
                        </tr>`;
                        tbody.innerHTML += row;
                    });
                } catch(e) { console.error(e); }
                setTimeout(poll, 100);
            }
            window.onload = poll;
        </script>
    </head>
    <body>
        <h1>Lynx vMix Bridge - Live</h1>
        <div id="time">Running Time</div>
        <table>
            <thead>
                <tr>
                    <th>Place</th>
                    <th>Lane</th>
                    <th>ID</th>
                    <th>Name</th>
                    <th>Affiliation</th>
                    <th>Time</th>
                </tr>
            </thead>
            <tbody id="results"></tbody>
        </table>
        <p><a href="/races" style="color: #0f0">View Races JSON</a></p>
    </body>
    </html>
    "#)
}

async fn get_live(State(state): State<SharedState>) -> Json<Value> {
    let s = state.read();
    let response = serde_json::json!([{
        "time": s.time,
        "running": s.running,
        "results": s.results,
        "messages": s.messages,
        "event_name": s.event_name,
        "event_number": s.event_number
    }]);
    Json(response)
}

async fn get_races(State(state): State<SharedState>) -> Json<Value> {
    let s = state.read();
    let races: Vec<_> = s.races.values().collect();
    Json(serde_json::to_value(races).unwrap())
}
