use image::{
    DynamicImage, EncodableLayout, GenericImage, GenericImageView, GrayImage, Luma, LumaA, Rgb,
    RgbImage, SubImage,
};
use plotters::prelude::*;
use std::fmt::Write;
use image::imageops::FilterType;
use vek::Vec2;
use num_traits::identities::Zero;
use system_greedy::generators::LatticeGenerator;
use system_greedy::system::System;
use system_greedy::utils::generate_arrow;

fn main() {
    let left = 25.0;
    let top = 27.0;
    let right = 2.0;
    let bottom = 15.0;

    // Открытие изображение
    let source_image = image::open("input/trim.png").unwrap();
    // В ргб
    let image = source_image.to_rgb8();
    // в серые тона
    let gray_image = source_image.to_luma8();

    // Создаем решетку
    let mut system = LatticeGenerator::trimer(450.0 / 2.0, 625.0, 20, 15);
    // Сохраняем решетку (не нужно тут)
    system.save_mfsys("results/trim.mfsys");

    // клонируем систему в другую переменную
    let mut system2 = system.clone();
    // клонируем список частиц
    let elements = system.elements().clone();

    // определяем максимальную координату х
    let max_x = elements.iter().max_by_key(|e| e.pos.x).map(|e| e.pos.x).unwrap();
    // определяем максимальную координату у
    let max_y = elements.iter().max_by_key(|e| e.pos.y).map(|e| e.pos.y).unwrap();

    // множитель для х
    let fx = (image.width() as f64 - left - right) / max_x.0;
    // множитель для у
    let fy = (image.height() as f64 - top - bottom) / max_y.0;

    let sx = 1000. / image.width() as f32;
    let sy = 1000. / image.height() as f32;

    let image = image::imageops::resize(&image, 1000, 1000, FilterType::Nearest);

    // создаем штуку для рисования
    let mut plotter = BitMapBackend::new("results/trim1.png", image.dimensions());
    // рисуем снимок на штуку
    plotter.blit_bitmap((0, 0), image.dimensions(), image.as_bytes()).unwrap();

    let mut plotter = plotter.into_drawing_area();

    // пробегаемся по всем частицам
    for (i, element) in elements.into_iter().enumerate() {
        // координаты частицы на изображении
        let x = (element.pos.x.0 * fx + left) as i32;
        let y = (element.pos.y.0 * fy + top) as i32;

        // суммируем для среднего
        let mut sum = 0;
        for oy in -1..=1 {
            for ox in -1..=1 {
                sum += gray_image[((x + ox) as u32, (y + oy) as u32)].0[0] as u32;
            }
        }

        // получаем среднее
        let mean = (sum / 9) as u8;

        let direction = element.magn.map(|x| x.0);

        // если в среднем белее
        if mean > 127 {
            // ставим положение спина в системе вверх, а во второй вниз
            set_spin_direction(&mut system, i, Direction::Up);
            set_spin_direction(&mut system2, i, Direction::Down);

            let direction = match get_spin_default_direction(&system, i) {
                Direction::Up => direction,
                Direction::Down => direction * -1.
            };

            plotter.draw(
                &(EmptyElement::at(((x as f32 * sx) as i32, (y as f32 * sy) as i32))
                    + generate_arrow(19., 3., 7., 10., direction, 0.8, &RED))
            ).unwrap();
        } else {
            // ставим положение спина в системе вниз, а во второй вверх
            set_spin_direction(&mut system, i, Direction::Down);
            set_spin_direction(&mut system2, i, Direction::Up);

            let direction = match get_spin_default_direction(&system, i) {
                Direction::Up => direction * -1.,
                Direction::Down => direction,
            };

            plotter.draw(
                &(EmptyElement::at(((x as f32 * sx) as i32, (y as f32 * sy) as i32))
                    + generate_arrow(19., 3., 7., 10., direction, 0.8, &GREEN))
            ).unwrap();
        }
    }

    // сохраняем штуку в картинку
    plotter.present().unwrap();


    // сохраняем системы
    system.save_mfsys("results/trim_1.mfsys");
    system2.save_mfsys("results/trim_2.mfsys");
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
}

fn get_spin_default_direction(system: &System, index: usize) -> Direction {
    let element = system.elements()[index];

    if element.pos.y.is_zero() {
        if element.magn.x.is_sign_negative() {
            Direction::Up
        } else {
            Direction::Down
        }
    } else {
        if element.magn.y.is_sign_negative() {
            Direction::Up
        } else {
            Direction::Down
        }
    }
}

fn set_spin_direction(system: &mut System, index: usize, direction: Direction) {
    let default_direction = get_spin_default_direction(system, index);

    if default_direction != direction {
        system.set_spin(index, true)
    } else {
        system.set_spin(index, false)
    }
}