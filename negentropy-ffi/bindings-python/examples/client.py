from negentropy import Negentropy, Bytes

# Client init
client = Negentropy(16, None)
client.add_item(0, Bytes.from_hex("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
client.add_item(1, Bytes.from_hex("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"))
client.seal()
init_output = client.initiate()
print(f"Initiator Output: {init_output.as_hex()}")

# Relay
relay = Negentropy(16, None)
relay.add_item(0, Bytes.from_hex("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
relay.add_item(2, Bytes.from_hex("cccccccccccccccccccccccccccccccc"))
relay.add_item(3, Bytes.from_hex("11111111111111111111111111111111"))
relay.add_item(5, Bytes.from_hex("22222222222222222222222222222222"))
relay.add_item(10, Bytes.from_hex("33333333333333333333333333333333"))
relay.seal()
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