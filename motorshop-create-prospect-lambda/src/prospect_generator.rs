use std::fs::File;
use std::io::BufWriter;

use printpdf::*;
use tracing::info;

use motorshop_domain::prospect::Prospect;

const FONT_BYTES: &[u8] = include_bytes!("../fonts/CastoroTitling-Regular.ttf");

pub fn generate_prospect(prospect: Prospect) -> String {
    info!("generate prospect...");
    let (doc, page1, layer1) =
        PdfDocument::new(format!("PROSPECT: {}", prospect.name),
                         Mm(247.0), Mm(210.0), "INTRO");

    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_external_font(FONT_BYTES).unwrap();
    current_layer.use_text(format!("PROSPECT"), 24.0, Mm(20.0), Mm(70.0), &font);
    current_layer.use_text(format!("------------------------------------------------"), 24.0, Mm(20.0), Mm(60.0), &font);
    current_layer.use_text(format!("BIKE:         {}", prospect.name), 24.0, Mm(20.0), Mm(30.0), &font);
    current_layer.use_text(format!("MODEL:    {}", prospect.model), 24.0, Mm(20.0), Mm(20.0), &font);

    let doc_name = format!("/tmp/{}-{}.pdf", prospect.name, prospect.model);
    doc.save(&mut BufWriter::new(File::create(doc_name).unwrap())).unwrap();

    return format!("/tmp/{}-{}.pdf", prospect.name, prospect.model);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_1() {
        let prospect = Prospect { name: "SuperBike".to_string(), model: "2022".to_string() };
        generate_prospect(prospect);
    }
}