.PHONY: init

# Link every component/app `docs/` folder under `codes/` into the root `docs/` directory.
init:
	@mkdir -p docs
	@found=0; \
	for docdir in $$(find codes \( -name target -o -name .git -o -name node_modules \) -prune -o -type d -name docs -print -prune | sort); do \
		project=$$(basename "$$(dirname "$$docdir")"); \
		link="docs/$$project"; \
		target="../$$docdir"; \
		if [ -e "$$link" ] && [ ! -L "$$link" ]; then \
			echo "skip: $$link exists and is not a symlink"; \
			continue; \
		fi; \
		ln -sfn "$$target" "$$link"; \
		echo "linked: $$link -> $$target"; \
		found=1; \
	done; \
	if [ "$$found" -eq 0 ]; then \
		echo "No docs folders found under codes/"; \
	fi
