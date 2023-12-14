from negentropy import Negentropy, NegentropyStorageVector, Bytes

# Client init
storage = NegentropyStorageVector()
storage.insert(0, Bytes.from_hex("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
storage.insert(1, Bytes.from_hex("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"))
storage.seal()
client = Negentropy(storage, None)
init_output = client.initiate()
print(f"Initiator Output: {init_output.as_hex()}")

# Relay
storage = NegentropyStorageVector()
storage.insert(0, Bytes.from_hex("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
storage.insert(2, Bytes.from_hex("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"))
storage.insert(3, Bytes.from_hex("1111111111111111111111111111111111111111111111111111111111111111"))
storage.insert(5, Bytes.from_hex("2222222222222222222222222222222222222222222222222222222222222222"))
storage.insert(10, Bytes.from_hex("3333333333333333333333333333333333333333333333333333333333333333"))
storage.seal()
relay = Negentropy(storage, None)
reconcile_output = relay.reconcile(init_output)
print(f"Reconcile Output: {reconcile_output.as_hex()}")

# Client reconcile
reconcile_output_with_ids = client.reconcile_with_ids(reconcile_output)
print(f"Reconcile Output with IDs: {reconcile_output_with_ids.output}")

print("Have IDs:")
for id in reconcile_output_with_ids.have_ids:
    print(f"- {id.as_hex()}")

print("Need IDs:")
for id in reconcile_output_with_ids.need_ids:
    print(f"- {id.as_hex()}")