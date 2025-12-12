.PHONY: all anchor_build update_idl copy_idl move_idl clean

# Build the IDL using Anchor, then post-process discriminators
all: idl

# Run anchor build to generate the IDL
anchor_build:
	cd src && anchor build --skip-lint

# Update discriminators to match the actual program
update_idl:
	cd src && bun ./scripts/update-discriminators.ts > ./target/idl/usdf_swap_updated.json

# Copy the updated IDL back to the target directory
copy_idl:
	cp -f ./src/target/idl/usdf_swap_updated.json ./src/target/idl/usdf_swap.json

# Move the final IDL to the output directory
move_idl:
	cp -f ./src/target/idl/usdf_swap_updated.json ./usdf_swap.json

# Full IDL generation pipeline
idl: anchor_build update_idl copy_idl move_idl
	@echo "IDL generated successfully at idl/usdf_swap.json"

# Clean build artifacts
clean:
	cd src && anchor clean
	rm -f usdf_swap.json
	rm -f ./src/target/idl/usdf_swap_updated.json
