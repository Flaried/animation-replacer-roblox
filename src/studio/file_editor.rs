use crate::StudioParser;
use rbx_binary::to_writer;
use std::fs::File;
use std::path::Path;

impl StudioParser {
    /// Saves the current DOM state to a .rbxl file
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut parser = StudioParser::from_rbxl("input.rbxl")?;
    /// // Make modifications...
    /// parser.save_to_rbxl("output.rbxl")?;
    /// ```

    pub fn save_to_rbxl<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let expanded_path = shellexpand::full(file_path.as_ref().to_str().unwrap())?;
        let file = File::create(expanded_path.as_ref())?;

        // Get the children of the root instead of the root
        let root_children = self.dom.get_by_ref(self.dom.root_ref()).unwrap().children();

        to_writer(file, &self.dom, root_children)?;
        Ok(())
    }
}
