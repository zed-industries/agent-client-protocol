package main

import (
	"flag"
	"os"
	"path/filepath"

	"github.com/zed-industries/agent-client-protocol/go/cmd/generate/internal/emit"
	"github.com/zed-industries/agent-client-protocol/go/cmd/generate/internal/load"
)

func main() {
	var schemaDirFlag string
	var outDirFlag string
	flag.StringVar(&schemaDirFlag, "schema", "", "path to schema directory (defaults to <repo>/schema)")
	flag.StringVar(&outDirFlag, "out", "", "output directory for generated go files (defaults to <repo>/go)")
	flag.Parse()

	repoRoot := findRepoRoot()
	schemaDir := schemaDirFlag
	outDir := outDirFlag
	if schemaDir == "" {
		schemaDir = filepath.Join(repoRoot, "schema")
	}
	if outDir == "" {
		outDir = filepath.Join(repoRoot, "go")
	}

	if err := os.MkdirAll(outDir, 0o755); err != nil {
		panic(err)
	}

	meta, err := load.ReadMeta(schemaDir)
	if err != nil {
		panic(err)
	}

	if err := emit.WriteConstantsJen(outDir, meta); err != nil {
		panic(err)
	}

	schema, err := load.ReadSchema(schemaDir)
	if err != nil {
		panic(err)
	}

	if err := emit.WriteTypesJen(outDir, schema, meta); err != nil {
		panic(err)
	}
	if err := emit.WriteDispatchJen(outDir, schema, meta); err != nil {
		panic(err)
	}
}

func findRepoRoot() string {
	cwd, _ := os.Getwd()
	dir := cwd
	for i := 0; i < 10; i++ {
		if _, err := os.Stat(filepath.Join(dir, "package.json")); err == nil {
			return dir
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	return cwd
}
