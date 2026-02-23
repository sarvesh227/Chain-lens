function switchTab(mode) {
    document.querySelectorAll(".tab").forEach(t => t.classList.remove("active"));
    document.querySelectorAll(".section").forEach(s => s.classList.remove("active"));

    if (mode === "tx") {
        document.querySelectorAll(".tab")[0].classList.add("active");
        document.getElementById("tx-section").classList.add("active");
    } else {
        document.querySelectorAll(".tab")[1].classList.add("active");
        document.getElementById("block-section").classList.add("active");
    }
}

async function analyzeTx() {
    const input = document.getElementById("tx-input").value;

    const res = await fetch("/api/analyze", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: input
    });

    const data = await res.json();

    if (!data.ok) {
        document.getElementById("tx-result").innerHTML =
            "<div class='warning'>" + data.error.message + "</div>";
        return;
    }

    renderTx(data);
}

function renderTx(tx) {

    let totalInput = tx.vin.reduce((a,i)=>a+i.prevout.value_sats,0);
    let totalOutput = tx.vout.reduce((a,o)=>a+o.value_sats,0);

    let html = "";

    /* WHAT HAPPENED */

    html += "<div class='card'><h2>What Happened?</h2>";
    html += "<p>This transaction spent <b>" + tx.vin.length +
        "</b> input(s) totaling <b>" + totalInput +
        "</b> sats and created <b>" + tx.vout.length +
        "</b> output(s) totaling <b>" + totalOutput +
        "</b> sats.</p>";
    html += "<p>The difference of <b>" + tx.fee_sats +
        "</b> sats was paid as a fee to miners.</p>";
    html += "<p><i>The outputs created here can later become inputs in future transactions.</i></p>";
    html += "</div>";

    /* SIZE & FEES */

    html += "<div class='card'><h2>Transaction Size & Fees</h2><div class='metrics'>";
    html += metric("Fee", tx.fee_sats + " sats");
    html += metric("Fee Rate", tx.fee_rate_sat_vb + " sat/vB");
    html += metric("Weight", tx.weight + " weight units");
    html += metric("Virtual Size", tx.vbytes + " vBytes");
    html += "</div>";
    html += "<p>Transactions take up space in a block. Larger transactions usually require higher fees.</p>";
    html += "</div>";

    /* VALUE FLOW */

    html += "<div class='card'><h2>Value Flow</h2><div class='flow'>";
    html += "<div class='column'>";
    tx.vin.forEach(i=>{
        html += "<div class='node input-node'>Input<br>"
            + i.prevout.value_sats + " sats</div>";
    });
    html += "</div>";

    html += "<div class='column'>";
    tx.vout.forEach(o=>{
        html += "<div class='node output-node'>Output<br>"
            + o.value_sats + " sats<br>"
            + o.script_type + "</div>";
    });
    html += "<div class='node fee-node'>Fee<br>"
        + tx.fee_sats + " sats</div>";
    html += "</div></div></div>";

    /* SEGWIT */

    html += "<div class='card'><h2>SegWit vs Legacy</h2>";
    html += "<p><b>txid:</b> " + tx.txid.substring(0,16) + "...</p>";
    html += "<p><b>wtxid:</b> " +
        (tx.wtxid ? tx.wtxid.substring(0,16)+"..." : "Same as txid (Legacy)") +
        "</p>";
    html += "<p>In legacy transactions both IDs are identical. In SegWit transactions, witness data changes the wtxid.</p>";
    html += "</div>";

    

    /* RBF */

    let rbf = tx.vin.some(i => i.sequence < 0xfffffffe);
    html += "<div class='card'><h2>Replace-By-Fee (RBF)</h2>";
    html += rbf
        ? "<p>This transaction signals RBF. The sender can increase the fee if it gets stuck.</p>"
        : "<p>This transaction does not signal RBF.</p>";
    html += "<p>RBF is controlled by the nSequence value in inputs.</p>";
    html += "</div>";

    /* TIMELOCKS */

    html += "<div class='card'><h2>Timelocks</h2>";
    html += (tx.locktime && tx.locktime !== 0)
        ? "<p>This transaction has an absolute timelock (locktime = " + tx.locktime + ").</p>"
        : "<p>No absolute timelock detected.</p>";
    html += "<p>Relative timelocks use sequence numbers inside inputs.</p>";
    html += "</div>";

    /* WARNINGS */

    if (tx.warnings && tx.warnings.length > 0) {
        html += "<div class='card'><h2>Warnings</h2>";
        tx.warnings.forEach(w=>{
            html += "<div class='warning'>" + w.code + "</div>";
        });
        html += "</div>";
    }

    /* TECHNICAL DETAILS */

    html += "<div class='card'>";
    html += "<div class='toggle' onclick='toggleTech()'>Show Technical Details</div>";
    html += "<div id='tech-section' class='hidden'>";
    html += "<pre>" + JSON.stringify(tx, null, 2) + "</pre>";
    html += "</div></div>";

    document.getElementById("tx-result").innerHTML = html;
}

function metric(title,value){
    return "<div class='metric'><div class='metric-title'>"
        + title + "</div><div class='metric-value'>"
        + value + "</div></div>";
}

function toggleTech(){
    const section = document.getElementById("tech-section");
    const button = document.querySelector(".toggle");

    section.classList.toggle("hidden");

    button.innerText = section.classList.contains("hidden")
        ? "Show Technical Details"
        : "Hide Technical Details";
}

function analyzeBlock(){
    alert("Connect backend for block analysis.");
}