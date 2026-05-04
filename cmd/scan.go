package cmd

import (
	"fmt"
	"wallman/internal/scanner"

	"github.com/spf13/cobra"
)

// scanCmd represents the scan command
var scanCmd = &cobra.Command{
	Use:   "scan [directory]",
	Short: "Scan a directory for image files",
	Long:  `Recursively scans the provided directory for .jpg, .jpeg, .png, and .webp files.`,
	Args:  cobra.MinimumNArgs(0),
	Run: func(cmd *cobra.Command, args []string) {

		path := args[0]
		images, err := scanner.Scan(path)
		if err != nil {
			fmt.Printf("Error scanning directory: %v\n", err)
			return
		}

		for _, img := range images {
			fmt.Println(img)
		}

		fmt.Printf("\nTotal images found: %d\n", len(images))
	},
}

func init() {
	rootCmd.AddCommand(scanCmd)
}
