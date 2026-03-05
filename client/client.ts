// =============================================================================
// Uma-Solana — Cliente para Solana Playground
// Pega este archivo en la pestaña "client.ts" de Playground y ejecútalo.
//
// INSTRUCCIONES DE USO:
//   1. Despliega el programa primero (botón Deploy).
//   2. Cambia ACTION y los parámetros según lo que quieras hacer.
//   3. Haz clic en "Run" para ejecutar.
// =============================================================================

// ── Configuración ─────────────────────────────────────────────────────────────
// Cambia esto para elegir la acción:
//   "info"       → Ver estado de tu Uma
//   "create"     → Crear una nueva Uma (solo la primera vez)
//   "train"      → Entrenar un stat
//   "rest"       → Descansar (recupera energía)
//   "recreation" → Recreación (mejora ánimo)
//   "race"       → Correr la carrera programada
const ACTION: string = "info";

// Parámetros según la acción:
const UMA_NAME: string  = "Mi Uma";          // Solo para "create"
const STAT_ID:  number  = 0;                 // Solo para "train": 0=Speed 1=Stamina 2=Power 3=Guts 4=Wit
// ─────────────────────────────────────────────────────────────────────────────

// Dirección del programa (cámbiala si redesplegaste)
const PROGRAM_ID = new web3.PublicKey(pg.program.programId);

// Derivar la PDA de la cuenta Uma del usuario
const [umaPda] = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("uma"), pg.wallet.publicKey.toBuffer()],
  PROGRAM_ID
);

console.log("=== Uma-Solana ===");
console.log("Wallet :", pg.wallet.publicKey.toString());
console.log("Uma PDA:", umaPda.toString());
console.log("Acción :", ACTION);
console.log("─────────────────────────────────────────");

// ── Helper: mostrar estado de la Uma ──────────────────────────────────────────
async function showUmaState() {
  let acc: any;
  try {
    acc = await pg.program.account.umaAccount.fetch(umaPda);
  } catch {
    console.log("❌  No se encontró una Uma. Usa ACTION = 'create' primero.");
    return;
  }
  const uma = acc.uma;
  const stats = uma.stats;

  const gradeName = (g: any) => Object.keys(g)[0].toUpperCase();
  const statLabel = (v: number) => {
    if (v <= 50)   return "G ";
    if (v <= 100)  return "G+";
    if (v <= 150)  return "F ";
    if (v <= 200)  return "F+";
    if (v <= 250)  return "E ";
    if (v <= 300)  return "E+";
    if (v <= 350)  return "D ";
    if (v <= 400)  return "D+";
    if (v <= 450)  return "C ";
    if (v <= 500)  return "C+";
    if (v <= 600)  return "B ";
    if (v <= 700)  return "B+";
    if (v <= 800)  return "A ";
    if (v <= 900)  return "A+";
    if (v <= 1000) return "S ";
    if (v <= 1100) return "S+";
    if (v <= 1200) return "SS";
    return "SS+";
  };

  console.log(`\n🐴  ${uma.name}`);
  console.log(`  Ánimo   : ${Object.keys(uma.mood)[0]}`);
  console.log(`  Energía : ${uma.energy}/100`);
  console.log(`  Turnos  : ${uma.turns}  |  Carreras: ${uma.races}  |  Victorias: ${uma.wins}`);
  console.log(`  Turnos hasta carrera: ${uma.turnsToRace}`);
  console.log(`  Fin de carrera: ${uma.isEnd}`);
  console.log(`\n  Stats:`);
  console.log(`    Speed   [${statLabel(stats.speed.value)}]  ${stats.speed.value}`);
  console.log(`    Stamina [${statLabel(stats.stamina.value)}]  ${stats.stamina.value}`);
  console.log(`    Power   [${statLabel(stats.power.value)}]  ${stats.power.value}`);
  console.log(`    Guts    [${statLabel(stats.guts.value)}]  ${stats.guts.value}`);
  console.log(`    Wit     [${statLabel(stats.wit.value)}]  ${stats.wit.value}`);
  console.log(`\n  Aptitudes:`);
  console.log(`    Turf: ${gradeName(uma.track.turf)}  Dirt: ${gradeName(uma.track.dirt)}`);
  console.log(`    Sprint: ${gradeName(uma.distance.sprint)}  Mile: ${gradeName(uma.distance.mile)}  Medium: ${gradeName(uma.distance.medium)}  Long: ${gradeName(uma.distance.long)}`);
  console.log(`    Front: ${gradeName(uma.style.front)}  Pace: ${gradeName(uma.style.pace)}  Late: ${gradeName(uma.style.late)}  End: ${gradeName(uma.style.end)}`);
  console.log(`    Estilo elegido: ${Object.keys(uma.chosenStyle)[0]}`);
}

// ── Ejecutar acción ───────────────────────────────────────────────────────────
try {
  if (ACTION === "info") {
    await showUmaState();

  } else if (ACTION === "create") {
    const tx = await pg.program.methods
      .createUma(UMA_NAME)
      .accounts({
        owner:      pg.wallet.publicKey,
        umaAccount: umaPda,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log("✅  Uma creada! Tx:", tx);
    await showUmaState();

  } else if (ACTION === "train") {
    const statNames = ["Speed","Stamina","Power","Guts","Wit"];
    console.log(`Entrenando: ${statNames[STAT_ID] ?? "Desconocido"}`);
    const tx = await pg.program.methods
      .train(STAT_ID)
      .accounts({
        owner:      pg.wallet.publicKey,
        umaAccount: umaPda,
      })
      .rpc();
    console.log("✅  Entrenamiento completado! Tx:", tx);
    await showUmaState();

  } else if (ACTION === "rest") {
    const tx = await pg.program.methods
      .rest()
      .accounts({
        owner:      pg.wallet.publicKey,
        umaAccount: umaPda,
      })
      .rpc();
    console.log("✅  Descansaste! Tx:", tx);
    await showUmaState();

  } else if (ACTION === "recreation") {
    const tx = await pg.program.methods
      .recreation()
      .accounts({
        owner:      pg.wallet.publicKey,
        umaAccount: umaPda,
      })
      .rpc();
    console.log("✅  Recreación completada! Tx:", tx);
    await showUmaState();

  } else if (ACTION === "race") {
    console.log("🏁  Iniciando carrera...");
    const tx = await pg.program.methods
      .race()
      .accounts({
        owner:      pg.wallet.publicKey,
        umaAccount: umaPda,
      })
      .rpc();
    console.log("✅  Carrera completada! Tx:", tx);
    console.log("📋  Ver resultados en los logs:", `https://explorer.solana.com/tx/${tx}?cluster=devnet`);
    await showUmaState();

  } else {
    console.log("❌  ACTION no reconocida:", ACTION);
    console.log("   Opciones: info | create | train | rest | recreation | race");
  }
} catch (err: any) {
  if (err?.logs) {
    console.error("❌  Error:", err.message);
    console.error("Logs del programa:");
    err.logs.forEach((l: string) => console.error(" ", l));
  } else {
    console.error("❌  Error:", err);
  }
}
