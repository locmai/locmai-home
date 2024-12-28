default:
	yarn start

OUT_DIR=resume
.PHONY: cv
cv:
	md2pdf --input ./$(OUT_DIR)/index.md --output ./$(OUT_DIR)/locmai-cv.pdf --metadata ./$(OUT_DIR)/metadata.yaml

shell:
	nix develop
