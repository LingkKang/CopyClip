pub mod cache;
pub mod monitor;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlite::{State, Value};
use tauri::{AppHandle, Manager, ClipboardManager};

use crate::config::ConfigMutex;

#[derive(Debug, Clone, Serialize, Deserialize,Default)]
pub struct Clip{
      pub text: String, // the text of the clip
      pub timestamp: i64, // in seconds
      pub id: i64, // the id of the clip
      pub favorite: bool, // if the clip is a favorite
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clips{
      pub current_clip: i64, // the id of the current clip
      pub current_page: i64, // the current page
      pub whole_list_of_ids: Vec<i64>, // the ids of all the clips, well sorted
      pub tray_ids_map: Vec<i64>, // the ids of the current displaying clips, the same order with the order displaying in the tray
      cached_clips: HashMap<i64,ClipCache>, // the clips that are currently in the cache
}

impl Default for Clips {
      fn default() -> Self {
            Self { 
                  current_clip: -1, 
                  current_page: 0,
                  whole_list_of_ids: Default::default(), 
                  tray_ids_map:Default::default(),
                  cached_clips: Default::default() 
            }
      }
}

#[derive(Default)]
pub struct ClipData {
      pub clips: Clips, // the clips
      database_connection: Option<sqlite::Connection>, // the connection to the database
}

pub struct ClipDataMutex {
      pub clip_data: std::sync::Mutex<ClipData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipCache{
      pub clip: Clip, // the clip
      pub add_timestamp: i64, // in seconds

      // cache management
      // load the latest config.clips_to_show*2 clips into the cache
      // if the cache is full, remove the oldest clips
      // if a clip that is not in the latest config.clips_to_show*2 clips, and not used in the last config.clip_cache_time(default a day) seconds, remove it from the cache
      // need a daemon thread to do this
}

impl ClipData {
      pub fn get_clip_pos(&self, id: i64) -> i64 {
            // get the position of the clip in the whole_list_of_ids
            // if the id is not in the list, return the highest pos
            let clip_pos = self.get_id_pos_in_whole_list_of_ids(id);
            if clip_pos.is_none() {
                  return self.clips.whole_list_of_ids.len() as i64 -1;
            }

            clip_pos.unwrap()
      }

      pub fn get_current_clip_pos(&self) -> i64 {
            self.get_clip_pos(self.clips.current_clip)
      }

      pub fn switch_page(&mut self, page: i64, max_page : i64) {
            // switch to the page by the page number
            // if max_page is -1, it means there is no limit

            let target_page = self.clips.current_page + page;
            if target_page < 0 {
                  self.clips.current_page = 0;
                  return;
            }
            if target_page > max_page && max_page >= 0{
                  self.clips.current_page = max_page;
                  return;
            }

            self.clips.current_page = target_page;
      }

      pub fn first_page(&mut self) {
            // switch to the first page

            self.clips.current_page = 0;
      }

      pub fn next_page(&mut self, app: &AppHandle) {
            // switch to the next page
            // if max_page is -1, it means there is no limit

            let max_page = self.get_max_page(app);
            self.switch_page(1, max_page);
      }

      pub fn prev_page(&mut self, app: &AppHandle) {
            // switch to the previous page
            // if max_page is -1, it means there is no limit

            let max_page = self.get_max_page(app);
            self.switch_page(-1, max_page);
      }

      pub fn get_max_page(&self, app: &AppHandle) -> i64 {
            // get the max page number
            // if there is no limit, return -1

            let config = app.state::<ConfigMutex>();
            let config = config.config.lock().unwrap();
            let max_page = self.clips.whole_list_of_ids.len() as i64 / config.clips_to_show;
            if max_page * config.clips_to_show == self.clips.whole_list_of_ids.len() as i64 {
                  return max_page -1;
            }
            max_page
      }

      pub fn get_id_pos_in_whole_list_of_ids(&self, id: i64) -> Option<i64> {
            // get the position of the id in the whole list of ids
            // if the id is not in the list, return None
            // use binary search
            
            let pos = self.clips.whole_list_of_ids.binary_search(&id);
            if pos.is_err() {
                  return None;
            }
            
            return Some(pos.unwrap() as i64);
      }

      pub fn get_clip(&mut self, mut id: i64) -> Result<Clip,String> {
            // if id is -1, change it to the latest clip
            if id == -1 {
                  let t = self.clips.whole_list_of_ids.last();
                  if t.is_none() {
                        return  Err("Clip not found for id: ".to_string() + &id.to_string());
                  }
                  let t = t.unwrap();
                  id = (*t).clone();
                  if id < 0 {
                        return  Err("Clip not found for id: ".to_string() + &id.to_string());
                  }
            }

            // if the clip is in the cache, return it
            let clip_cache = self.clips.cached_clips.get(&id);
            if clip_cache.is_some() {
                  let clip_cache = clip_cache.unwrap();
                  let clip_cache = clip_cache.clone();
                  self.clips.cached_clips.insert(id, ClipCache{
                        clip: clip_cache.clip.clone(),
                        add_timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                  });
                  return Ok(clip_cache.clip);
            }

            // if the clip is not in the cache, get it from the database
            let mut statement = self.database_connection.as_ref().unwrap().prepare("SELECT * FROM clips WHERE id = ?").unwrap();
            statement.bind((1,id)).unwrap();
            loop {
                  let state = statement.next();
                  if state.is_err() {
                        return Err(state.err().unwrap().message.unwrap());
                  }
                  let state = state.unwrap();
                  if state == State::Done {
                        break;
                  }

                  let text = statement.read::<String,_>("text");
                  if text.is_err() {
                        return Err(text.err().unwrap().message.unwrap());
                  }

                  let timestamp = statement.read::<i64,_>("timestamp");
                  if timestamp.is_err() {
                        return Err(timestamp.err().unwrap().message.unwrap());
                  }

                  let id = statement.read::<i64,_>("id");
                  if id.is_err() {
                        return Err(id.err().unwrap().message.unwrap());
                  }
                  let id = id.unwrap();

                  let favorite = statement.read::<i64,_>("favorite");
                  if favorite.is_err() {
                        return Err(favorite.err().unwrap().message.unwrap());
                  }
                  let favorite = favorite.unwrap() == 1;

                  let clip = Clip{
                        text: text.unwrap(),
                        timestamp: timestamp.unwrap(),
                        id,
                        favorite,
                  };

                  self.clips.cached_clips.insert(id, ClipCache{
                        clip: clip.clone(),
                        add_timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                  });

                  return Ok(clip);
            }

            // if the clip is not in the database, return None
            Err("Clip not found for id: ".to_string() + &id.to_string())
      }

      pub fn get_current_clip(&mut self) -> Result<Clip,String> {
            self.get_clip(self.clips.current_clip)
      }

      pub fn delete_clip(&mut self, id: i64) -> Result<(),String> {
            // delete a clip from the database and the cache

            // first delete in cache
            self.clips.cached_clips.remove(&id);

            // delete in database
            let mut statement = self.database_connection.as_ref().unwrap().prepare("DELETE FROM clips WHERE id = ?").unwrap();
            statement.bind((1,id)).unwrap();
            if let Ok(State::Done) = statement.next() {
                  // delete from the whole list of ids
                  // get the position of the id in the whole list of ids
                  let pos = self.get_id_pos_in_whole_list_of_ids(id);
                  let current_clip_pos = self.get_id_pos_in_whole_list_of_ids(self.clips.current_clip);
                  if pos.is_some() {
                        let pos = pos.unwrap();
                        // if pos is before the current clip, decrease the current clip by 1
                        // if pos is the current clip, set the current clip to -1
                        if current_clip_pos.is_none() {
                              self.clips.current_clip = -1;
                        } else {
                              let current_clip_pos = current_clip_pos.unwrap();
                              if pos < current_clip_pos {
                                    self.clips.current_clip = self.clips.whole_list_of_ids.get(current_clip_pos as usize - 1).unwrap().clone();
                              } else if pos == current_clip_pos {
                                    self.clips.current_clip = -1;
                              }
                        }
                        self.clips.whole_list_of_ids.remove(pos.try_into().unwrap());
                  }
                  return Ok(());
            }

            Err("Failed to delete clip".to_string())
      }

      pub fn new_clip(&mut self, text: String) -> Result<i64,String> {
            // create a new clip in the database and return the id of the new clip

            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;

            let connection = &self.database_connection;
            if connection.is_none() {
                  return Err("Failed to get database connection".to_string());
            }
            let connection = connection.as_ref().unwrap();
            let mut statement = connection.prepare("INSERT INTO clips (text, timestamp, favorite) VALUES (?, ?, ?)").unwrap();
            statement.bind::<&[(_, Value)]>(&[
                  (1, text.clone().into()),
                  (2, timestamp.into()),
                  (3, 0.into()),
            ][..]).unwrap();
            if let Ok(State::Done) = statement.next() {
                  // try to get the id of the new clip by searching for the clip with the same timestamp
                  let mut statement = self.database_connection.as_ref().unwrap().prepare("SELECT * FROM clips WHERE timestamp = ?").unwrap();
                  statement.bind((1,timestamp)).unwrap();
                  while let Ok(State::Row) = statement.next() {
                        let id = statement.read::<i64,_>("id");
                        if id.is_err() {
                              return Err("Failed to get id of new clip".to_string());
                        }
                        let id = id.unwrap();

                        let clip = Clip{
                              text,
                              timestamp,
                              id,
                              favorite: false,
                        };

                        self.clips.cached_clips.insert(id, ClipCache{
                              clip: clip,
                              add_timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                        });

                        self.clips.whole_list_of_ids.push(id);

                        // change the current clip to the last one
                        self.clips.current_clip = id;

                        return Ok(id);
                  }
            }

            Err("Failed to create new clip".to_string())

      }

      pub fn toggle_favorite_clip(&mut self, _id: i64) -> Result<bool,String> {
            // toggle the favorite status of a clip
            // if the clip is not in the cache, return an error
            // return the new favorite status


            !todo!("toggle_favorite_clip")
      }

      pub fn select_clip(&mut self, app: &AppHandle, id: i64) -> Result<(),String> {
            // select a clip by id
            // change the current clip to the clip, and fill in the text of the clip to the clipboard

            // try get the clip
            let c = self.get_clip(id);
            if c.is_err() {
                  return Err(c.err().unwrap());
            }

            let c = c.unwrap();
            self.clips.current_clip = id;
            
            let mut clipboard_manager = app.clipboard_manager();
            let res = clipboard_manager.write_text(c.text);
            if res.is_err() {
                  return Err("Failed to write to clipboard".to_string());
            }

            Err("Clip not found".to_string())
      }

      pub fn update_tray(&mut self, app: &AppHandle) -> Result<(),String> {
            // get the clips per page configuration
            let config = app.state::<ConfigMutex>();
            let config = config.config.lock().unwrap();
            let clips_per_page = config.clips_to_show;
            let max_clip_length = config.clip_max_show_length;
            drop(config);

            // get the current page
            let mut current_page = self.clips.current_page;
            let whole_pages = self.get_max_page(app);
            // if the current page bigger than the whole pages, then calculate the current page, regarding to current_clip_pos
            if current_page > whole_pages {
                  // get the current clip pos
                  let current_clip_pos: i64 = self.get_current_clip_pos();

                  current_page = (self.clips.whole_list_of_ids.len() as i64 - current_clip_pos - 1) / clips_per_page;
                  self.clips.current_page = current_page;
            }

            // get the current page clips
            let mut current_page_clips = Vec::new();
            for i in 0..clips_per_page {
                  let pos = self.clips.whole_list_of_ids.len() as i64 - (current_page * clips_per_page + i + 1);
                  if pos< 0 {
                        break;
                  }
                  let clip_id = self.clips.whole_list_of_ids.get(pos as usize);
                  if clip_id.is_none() {
                        break;
                  }
                  let clip_id = clip_id.unwrap();
                  let clip = self.get_clip(*clip_id);
                  if clip.is_err() {
                        return Err(clip.err().unwrap());
                  }
                  let clip = clip.unwrap();
                  current_page_clips.push(clip);
            }

            // put text in
            // empty the tray_ids_map
            self.clips.tray_ids_map.clear();
            for i in 0..current_page_clips.len() {
                  let tray_id = "tray_clip_".to_string() + &i.to_string();
                  let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);
                  let res = tray_clip_sub_menu.set_title(trim_clip_text(current_page_clips.get(i).unwrap().text.clone(), max_clip_length));
                  if res.is_err() {
                        return Err("Failed to set tray clip sub menu title".to_string());
                  }
                  self.clips.tray_ids_map.push(current_page_clips.get(i).unwrap().id.clone())
            }

            // clean out the rest of the tray
            for i in current_page_clips.len()..clips_per_page as usize {
                  let tray_id = "tray_clip_".to_string() + &i.to_string();
                  let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);
                  let res = tray_clip_sub_menu.set_title("".to_string());
                  if res.is_err() {
                        return Err("Failed to set tray clip sub menu title".to_string());
                  }
            }

            let tray_page_info_item = app.tray_handle().get_item("page_info");
            // TODO change the method of doing this to a more clean one
            let tray_page_info_title = "Total clips: ".to_string() + &self.clips.whole_list_of_ids.len().to_string()+", Current page: " + &(&current_page+1).to_string()+"/" + &(whole_pages + 1).to_string();
            let res = tray_page_info_item.set_title(tray_page_info_title);
            if res.is_err() {
                  // TODO change this constant error message to constant
                  return Err("Failed to set page info title".to_string());
            }

            Ok(())
      }

}

pub fn trim_clip_text(text: String, l: i64) -> String {
      // trim the leading white space
      let text = text.trim_start().to_string();

      if l < 3 {
            return text;
      }
      if text.len() as i64 <= l {
            return text;
      }
      let mut res = String::new();
      for i in 0..(l-3) {
            res.push(text.chars().nth(i as usize).unwrap());
      }
      res.push_str("...");
      res
}

pub fn init_database_connection(app: &AppHandle) -> Result<(), String> {
      // get the app data dir
      let app_data_dir = app.path_resolver().app_data_dir();
      if app_data_dir.is_none() {
            return Err("Failed to get app data dir".to_string());
      }
      let mut app_data_dir = app_data_dir.unwrap();

      // if the app data dir does not exist, create it
      if app_data_dir.exists() == false {
            if let Err(_) = std::fs::create_dir_all(&app_data_dir) {
                  return Err("Failed to create app data dir".to_string());
            }
      }

      // create the database dir if it does not exist
      app_data_dir.push("database");
      let database_dir = app_data_dir; // /Users/zifanhua/Library/Application Support/org.eu.huazifan.copyclip/database
      
      let connection = sqlite::open(database_dir.as_path());
      if connection.is_err() {
            return Err("Failed to open database".to_string());
      }

      let connection = connection.unwrap();

      // create the clips table if it does not exist
      let mut statement = connection.prepare("CREATE TABLE IF NOT EXISTS clips (id INTEGER PRIMARY KEY AUTOINCREMENT, text TEXT, timestamp INTEGER, favorite INTEGER)").unwrap();
      let state = statement.next();
      if let Ok(State::Done) = state {
            let clip_data_mutex = app.state::<ClipDataMutex>();
            let mut clip_data = clip_data_mutex.clip_data.lock().unwrap();
            drop(statement);
            clip_data.database_connection = Some(connection);

            // get the whole clips ids
            let mut ids = Vec::new();
            let mut statement = clip_data.database_connection.as_ref().unwrap().prepare("SELECT id FROM clips").unwrap();
            while let Ok(State::Row) = statement.next() {
                  let id = statement.read::<i64,_>("id");
                  if id.is_err() {
                        return Err("Failed to get id of clip".to_string());
                  }
                  let id = id.unwrap();

                  ids.push(id);
            }
            drop(statement);
            clip_data.clips.whole_list_of_ids = ids;

            return Ok(());
      } else if state.is_err() {
            return Err("Failed to create clips table: ".to_string() + &state.err().unwrap().message.unwrap().to_string());
      }

      return Err("Failed to create clips table".to_string());
}