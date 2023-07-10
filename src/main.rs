use bitcoin::consensus::Decodable;

fn get_block(hash: &'static str) -> bitcoin::Block {
    let block_serialization = std::process::Command::new("bitcoin-cli")
        .arg("getblock")
        .arg(hash)
        .arg("0")
        .output()
        .expect("failed to execute process")
        .stdout;
    let string_stdout = String::from_utf8(block_serialization).unwrap();
    let mut serialization = std::io::Cursor::new(hex::decode(string_stdout.trim()).unwrap());
    bitcoin::Block::consensus_decode(&mut serialization).unwrap()
}

fn find_tx<'a>(block: &'a bitcoin::Block, txid: &'static str) -> &'a bitcoin::Transaction {
    block
        .txdata
        .iter()
        .filter(|tx| tx.txid().to_string() == txid)
        .collect::<Vec<&bitcoin::Transaction>>()
        .first()
        .unwrap()
}

fn main() {
    // Which transaction sticks out?
    // https://mempool.space/block/00000000000000ecbbff6bafb7efa2f7df05b227d5c73dca8f2635af32a2e949
    let block = get_block("00000000000000ecbbff6bafb7efa2f7df05b227d5c73dca8f2635af32a2e949");
    let tx: &bitcoin::Transaction = find_tx(
        &block,
        // Which outputs stick out?
        // https://mempool.space/tx/54e48e5f5c656b26c3bca14a8c95aa583d07ebe84dde3b7dd4a78f4e4186e713
        "54e48e5f5c656b26c3bca14a8c95aa583d07ebe84dde3b7dd4a78f4e4186e713",
    );
    let mut all_bytes: Vec<u8> = vec![];
    for output in &tx.output {
        // How should we filter for interesting outputs?
        // https://blockstream.info/tx/54e48e5f5c656b26c3bca14a8c95aa583d07ebe84dde3b7dd4a78f4e4186e713?expand
        if output.script_pubkey.instructions().last().unwrap().unwrap()
            != bitcoin::script::Instruction::Op(bitcoin::blockdata::opcodes::all::OP_CHECKMULTISIG)
        {
            continue;
        }
        for instruction in output.script_pubkey.instructions() {
            // What are those data pushes encoding?
            // echo "first_data_push_of_first_output" | xxd -p -r
            if let bitcoin::blockdata::script::Instruction::PushBytes(pb) = instruction.unwrap() {
                all_bytes.extend(pb.as_bytes());
            }
        }
    }
    std::fs::write("mystery.file", all_bytes).expect("Unable to write file");
}
