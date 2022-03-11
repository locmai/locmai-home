OUT_DIR=resume
.PHONY: resume
resume:
	md2pdf --input ./$(OUT_DIR)/index.md --output ./$(OUT_DIR)/resume.pdf --metadata ./$(OUT_DIR)/metadata.yaml
