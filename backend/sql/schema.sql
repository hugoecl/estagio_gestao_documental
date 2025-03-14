CREATE TABLE IF NOT EXISTS users (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password BINARY(48) NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS user_page_analytics (
  id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
  user_id INT UNSIGNED NOT NULL,
  page_path VARCHAR(255) NOT NULL,
  visit_count INT UNSIGNED DEFAULT 1 NOT NULL,
  last_visited_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY unique_user_page (user_id, page_path),
  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS contracts (
  id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
  contract_number INT UNSIGNED NOT NULL,
  date DATE NOT NULL,
  date_start DATE NOT NULL,
  date_end DATE NOT NULL,
  description TEXT NOT NULL,
  location TINYINT NOT NULL COMMENT '0: Viana do Castelo, 1: Braga, 2: Porto, 3: Vila Real',
  service TINYINT NOT NULL COMMENT '0: Electricity, 1: Water, 2: Cleaning, 3: Printers, 4: Comunications',
  status TINYINT NOT NULL COMMENT '0: Active, 1: Inactive',
  supplier VARCHAR(100) NOT NULL,
  type TINYINT NOT NULL COMMENT '0: Addendum, 1: New, 2: Renew',
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS contract_files (
  id INT UNSIGNED NOT NULL AUTO_INCREMENT,
  contract_id INT UNSIGNED NOT NULL,
  file_path VARCHAR(255) NOT NULL COMMENT 'Is relative to the backend folder includes the file name',
  uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  FOREIGN KEY (contract_id) REFERENCES contracts (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS work_contract_categories (
  id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
  name VARCHAR(100) NOT NULL,
  description TEXT,
  PRIMARY KEY (id),
  UNIQUE KEY unique_category_name (name)
);

CREATE TABLE IF NOT EXISTS work_contracts (
  id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
  employee_name VARCHAR(255) NOT NULL,
  nif VARCHAR(20) NOT NULL,
  start_date DATE NOT NULL,
  end_date DATE,
  type TINYINT NOT NULL COMMENT '0: Adenda, 1: Contrato de Funcionario',
  location TINYINT NOT NULL COMMENT '0: Viana do Castelo, 1: Braga, 2: Porto, 3: Vila Real',
  category_id INT UNSIGNED NOT NULL,
  description TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  FOREIGN KEY (category_id) REFERENCES work_contract_categories (id)
);

CREATE TABLE IF NOT EXISTS work_contract_files (
  id INT UNSIGNED NOT NULL AUTO_INCREMENT,
  contract_id INT UNSIGNED NOT NULL,
  file_path VARCHAR(255) NOT NULL COMMENT 'Is relative to the backend folder includes the file name',
  uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  FOREIGN KEY (contract_id) REFERENCES work_contracts (id) ON DELETE CASCADE
);