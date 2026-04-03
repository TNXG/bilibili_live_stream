use crate::api::area::fetch_area_list;
use crate::error::Result;
use crate::{user_info, user_input_prompt, user_success, user_warning};

pub fn get_area_choice() -> Result<u32> {
    let area_list = fetch_area_list()?;

    loop {
        user_info!("一级分区列表:");
        if let Some(data) = area_list["data"].as_array() {
            for (i, area) in data.iter().enumerate() {
                user_info!("{}. {}", i + 1, area["name"]);
            }
        }

        user_input_prompt!("请输入一级分区编号: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let first_choice: usize = input.trim().parse()?;

        if first_choice == 0 {
            user_warning!("你是抱着多大的觉悟在一级菜单按下0的？");
            continue;
        }

        if let Some(data) = area_list["data"].as_array()
            && first_choice > 0
            && first_choice <= data.len()
        {
            let selected_first_area = &data[first_choice - 1];

            if let Some(second_list) = selected_first_area["list"].as_array() {
                loop {
                    user_info!("二级分区列表 ({}):", selected_first_area["name"]);
                    for (i, area) in second_list.iter().enumerate() {
                        user_info!("{}. {} - {}", i + 1, area["name"], area["id"]);
                    }

                    user_input_prompt!("请输入二级分区编号(输入0返回): ");
                    let mut second_input = String::new();
                    std::io::stdin().read_line(&mut second_input)?;
                    let second_choice: usize = second_input.trim().parse()?;

                    if second_choice == 0 {
                        break;
                    }

                    if second_choice > 0 && second_choice <= second_list.len() {
                        let selected_area = &second_list[second_choice - 1];
                        user_success!(
                            "已选择分区: {} (ID: {})",
                            selected_area["name"],
                            selected_area["id"]
                        );
                        let id_str = selected_area["id"].as_str().unwrap_or("");
                        let numeric_id: String =
                            id_str.chars().filter(|c| c.is_numeric()).collect();
                        return Ok(numeric_id.parse::<u32>()?);
                    }

                    user_warning!("无效的选择，请重新输入");
                }
            }
        }

        user_warning!("无效的选择，请重新输入");
    }
}
