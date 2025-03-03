CREATE TABLE IF NOT EXISTS users (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password BINARY(48) NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (id)
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
  PRIMARY KEY (id),
  UNIQUE INDEX idx_contract_number (contract_number),
  INDEX idx_supplier (supplier),
  INDEX idx_status (status),
  INDEX idx_type (type)
);

CREATE TABLE IF NOT EXISTS contract_files (
  id INT UNSIGNED NOT NULL AUTO_INCREMENT,
  contract_id INT UNSIGNED NOT NULL,
  file_path VARCHAR(255) NOT NULL COMMENT 'Is relative to the backend folder includes the file name',
  uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  INDEX idx_contract_id (contract_id),
  FOREIGN KEY (contract_id) REFERENCES contracts (id) ON DELETE CASCADE
);