use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use nalgebra::DMatrix;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([950.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "万能行列計算・可視化シミュレータ",
        options,
        Box::new(|cc| {
            // 👇 日本語フォントを設定する関数を呼び出す
            setup_japanese_font(&cc.egui_ctx);
            Box::new(MatrixApp::default())
        }),
    )
}

/// OSの日本語フォントを読み込んでeguiに設定するヘルパー関数
fn setup_japanese_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Windowsの「MSゴシック」やMacの「ヒラギノ」、Linuxの「Noto」などを探す設定
    // ※より確実にするには、プロジェクト内に「NotoSansJP-Regular.ttf」などのファイルを同梱して
    // `include_bytes!` で読み込むのがベストですが、まずはOSのフォントを試します。
    
    # [cfg(target_os = "windows")]
    let font_path = "C:\\Windows\\Fonts\\msgothic.ttc"; // Windowsの場合
    # [cfg(target_os = "macos")]
    let font_path = "/System/Library/Fonts/Hiragino Sans GB.ttc"; // Macの場合
    # [cfg(not(any(target_os = "windows", target_os = "macos")))]
    let font_path = "/usr/share/fonts/TRUETYPE/noto/NotoSansCJK-Regular.ttc"; // Linuxなどの想定

    // フォントファイルの読み込みに挑戦
    if let Ok(font_data) = std::fs::read(font_path) {
        fonts.font_data.insert(
            "japanese_font".to_owned(),
            egui::FontData::from_owned(font_data),
        );

        // プロポーショナル（通常テキスト）とモノスペース（等幅）の両方に日本語を最優先で割り当て
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "japanese_font".to_owned());

        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "japanese_font".to_owned());

        // 設定をコンテキストに反映
        ctx.set_fonts(fonts);
    }
}

#[derive(PartialEq, Copy, Clone)]
enum MatrixSize {
    TwoByTwo = 2,
    ThreeByThree = 3,
    FourByFour = 4,
}

struct MatrixApp {
    size: MatrixSize,
    matrix_a_input: [[String; 4]; 4],
    matrix_b_input: [[String; 4]; 4],
    mode: usize, // 0: 行列Aの分析, 1: 行列A × 行列B
}

impl Default for MatrixApp {
    fn default() -> Self {
        let mut app = Self {
            size: MatrixSize::TwoByTwo,
            matrix_a_input: Default::default(),
            matrix_b_input: Default::default(),
            mode: 0,
        };
        app.reset_matrices();
        app
    }
}

impl MatrixApp {
    fn reset_matrices(&mut self) {
        let n = self.size as usize;
        for r in 0..4 {
            for c in 0..4 {
                if r < n && c < n {
                    if r == c {
                        self.matrix_a_input[r][c] = "1.0".to_string();
                        self.matrix_b_input[r][c] = "1.0".to_string();
                    } else {
                        self.matrix_a_input[r][c] = "0.0".to_string();
                        self.matrix_b_input[r][c] = "0.0".to_string();
                    }
                } else {
                    self.matrix_a_input[r][c] = "".to_string();
                    self.matrix_b_input[r][c] = "".to_string();
                }
            }
        }
    }

    fn parse_matrix(&self, input: &[[String; 4]; 4]) -> Option<DMatrix<f64>> {
        let n = self.size as usize;
        let mut data = Vec::with_capacity(n * n);
        for r in 0..n {
            for c in 0..n {
                if let Ok(val) = input[r][c].parse::<f64>() {
                    data.push(val);
                } else {
                    return None;
                }
            }
        }
        Some(DMatrix::from_row_slice(n, n, &data))
    }

    fn show_matrix_grid(ui: &mut egui::Ui, input: &mut [[String; 4]; 4], n: usize, id: &str) {
        egui::Grid::new(id).spacing([8.0, 8.0]).show(ui, |ui| {
            for r in 0..n {
                for c in 0..n {
                    ui.add(egui::TextEdit::singleline(&mut input[r][c]).desired_width(55.0));
                }
                ui.end_row();
            }
        });
    }

    fn display_result_matrix(ui: &mut egui::Ui, mat: &DMatrix<f64>, id: &str) {
        let (rows, cols) = mat.shape();
        egui::Grid::new(id).spacing([15.0, 5.0]).show(ui, |ui| {
            for r in 0..rows {
                for c in 0..cols {
                    ui.colored_label(egui::Color32::LIGHT_GREEN, format!("{:.3}", mat[(r, c)]));
                }
                ui.end_row();
            }
        });
    }

    fn draw_transformation_plot(&self, ui: &mut egui::Ui, mat: &DMatrix<f64>) {
        Plot::new("spatial_transform")
            .view_aspect(1.0)
            .data_aspect(1.0)
            .include_x(2.0)
            .include_x(-2.0)
            .include_y(2.0)
            .include_y(-2.0)
            .allow_zoom(false)
            .allow_drag(false)
            .show(ui, |plot_ui| {
                for i in -2..=2 {
                    let val = i as f64;
                    plot_ui.line(Line::new(PlotPoints::new(vec![[val, -2.0], [val, 2.0]])).color(egui::Color32::from_gray(60)).width(1.0));
                    plot_ui.line(Line::new(PlotPoints::new(vec![[-2.0, val], [2.0, val]])).color(egui::Color32::from_gray(60)).width(1.0));
                }

                let a = mat[(0, 0)]; let b = mat[(0, 1)];
                let c = mat[(1, 0)]; let d = mat[(1, 1)];

                for i in -2..=2 {
                    let k = i as f64;
                    let mut v_points = Vec::new();
                    for y_idx in -20..=20 {
                        let y = (y_idx as f64) * 0.1;
                        v_points.push([a * k + b * y, c * k + d * y]);
                    }
                    plot_ui.line(Line::new(PlotPoints::new(v_points)).color(egui::Color32::from_rgba_unmultiplied(100, 200, 100, 180)).width(1.5));

                    let mut h_points = Vec::new();
                    for x_idx in -20..=20 {
                        let x = (x_idx as f64) * 0.1;
                        h_points.push([a * x + b * k, c * x + d * k]);
                    }
                    plot_ui.line(Line::new(PlotPoints::new(h_points)).color(egui::Color32::from_rgba_unmultiplied(100, 200, 100, 180)).width(1.5));
                }

                let mut circle_points = Vec::new();
                for theta_idx in 0..=100 {
                    let theta = (theta_idx as f64) * (std::f64::consts::TAU / 100.0);
                    let x = theta.cos();
                    let y = theta.sin();
                    let tx = a * x + b * y;
                    let ty = c * x + d * y;
                    circle_points.push([tx, ty]);
                }
                plot_ui.line(Line::new(PlotPoints::new(circle_points)).color(egui::Color32::LIGHT_BLUE).width(2.5));
            });
    }
}

impl eframe::App for MatrixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🧮 万能行列計算・可視化シミュレータ");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("⚙️ 行列のサイズ選択:");
                let old_size = self.size;
                if ui.radio_value(&mut self.size, MatrixSize::TwoByTwo, "2 x 2 (可視化対応)").changed() ||
                   ui.radio_value(&mut self.size, MatrixSize::ThreeByThree, "3 x 3").changed() ||
                   ui.radio_value(&mut self.size, MatrixSize::FourByFour, "4 x 4").changed() {
                    if old_size != self.size {
                        self.reset_matrices();
                    }
                }
                ui.add_space(20.0);
                if ui.button("データをリセット (単位行列化)").clicked() {
                    self.reset_matrices();
                }
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("🎯 計算モード:");
                ui.selectable_value(&mut self.mode, 0, "単一行列Aの特性（行列式・逆行列）");
                ui.selectable_value(&mut self.mode, 1, "2つの行列の積 (A × B)");
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            let n = self.size as usize;
            let mat_a_opt = self.parse_matrix(&self.matrix_a_input);
            let mat_b_opt = self.parse_matrix(&self.matrix_b_input);

            ui.columns(2, |panels| {
                // =============== 左パネル: 数値計算・入力 ===============
                panels[0].vertical(|ui| {
                    if self.mode == 0 {
                        ui.strong("【 行列 A の入力 】");
                        ui.add_space(5.0);
                        Self::show_matrix_grid(ui, &mut self.matrix_a_input, n, "grid_a_m0");
                        ui.add_space(15.0);

                        ui.strong("【 計算結果・幾何学的性質 】");
                        ui.add_space(5.0);
                        if let Some(mat_a) = &mat_a_opt {
                            let det = mat_a.determinant();
                            ui.horizontal(|ui| {
                                ui.label("🔴 行列式 (det A) = ");
                                ui.colored_label(egui::Color32::LIGHT_BLUE, format!("{:.4}", det));
                            });
                            
                            if det.abs() < 1e-6 {
                                ui.colored_label(egui::Color32::LIGHT_RED, "⚠️ 行列式が0です。空間が完全に潰れます。");
                            } else {
                                ui.label(format!("💡 空間のエリアが {:.2} 倍になります。", det.abs()));
                            }

                            ui.add_space(10.0);
                            ui.label("🔵 逆行列 (A⁻¹):");
                            if let Some(inv_a) = mat_a.clone().try_inverse() {
                                Self::display_result_matrix(ui, &inv_a, "inv_a_res");
                            } else {
                                ui.colored_label(egui::Color32::RED, "逆行列は存在しません。");
                            }
                        } else {
                            ui.colored_label(egui::Color32::RED, "⚠️ 有効な数値を入力してください。");
                        }
                    } else {
                        // モード1: 行列の積
                        ui.strong("【 行列A × 行列B 】");
                        ui.add_space(5.0);
                        ui.label("行列 A:");
                        Self::show_matrix_grid(ui, &mut self.matrix_a_input, n, "grid_a_m1");
                        ui.add_space(5.0);
                        ui.label("行列 B:");
                        Self::show_matrix_grid(ui, &mut self.matrix_b_input, n, "grid_b_m1");
                        
                        ui.add_space(15.0);
                        ui.strong("【 計算結果 (A × B) 】");
                        if let (Some(mat_a), Some(mat_b)) = (&mat_a_opt, &mat_b_opt) {
                            let mat_res = mat_a * mat_b;
                            Self::display_result_matrix(ui, &mat_res, "prod_res");
                            ui.add_space(5.0);
                            ui.label(format!("合成行列の行列式: {:.4}", mat_res.determinant()));
                            
                            // ★ エラー箇所を ui.weak に修正して解決
                            ui.weak("※ det(A×B) は、常に det(A) × det(B) と一致します。");
                        } else {
                            ui.colored_label(egui::Color32::RED, "⚠️ 数値のパースに失敗しました。");
                        }
                    }
                });

                // =============== 右パネル: 空間変形のリアルタイムプロット ===============
                panels[1].vertical(|ui| {
                    ui.strong("🌐 2D線形変換の可視化 (グリッド変形)");
                    ui.add_space(5.0);

                    if self.size != MatrixSize::TwoByTwo {
                        ui.colored_label(egui::Color32::YELLOW, "※ 空間プロットは 2x2 行列モードの時のみ描画されます。");
                        ui.label("3x3以上は高次元のため、左側の数値解析を参照してください。");
                    } else {
                        if self.mode == 0 {
                            if let Some(mat_a) = &mat_a_opt {
                                self.draw_transformation_plot(ui, mat_a);
                            }
                        } else {
                            if let (Some(mat_a), Some(mat_b)) = (&mat_a_opt, &mat_b_opt) {
                                let mat_res = mat_a * mat_b; 
                                self.draw_transformation_plot(ui, &mat_res);
                            }
                        }
                    }
                });
            });
        });
    }
}