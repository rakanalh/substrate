fn main() {
	tonic_build::compile_protos("proto/signer.proto").unwrap();
}
