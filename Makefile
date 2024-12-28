default:
	trunk serve

shell:
	nix develop

tailwind:
	tailwindcss -i ./style/input.css -o ./style/output.css

