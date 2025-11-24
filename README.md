# HealthFiti

HealthFiti is a comprehensive healthcare platform designed to streamline healthcare operations, improve patient management, and integrate telemedicine, e-commerce, and AI-driven solutions. It empowers health facilities, pharmacies, and patients with modern digital tools to enhance care delivery and efficiency.

---

## Features

### Core Modules
- **EHR (Electronic Health Records)**: Manage patient records, appointments, and history securely.
- **Telemedicine**: Video consultations, patient messaging, and remote health monitoring.
- **Pharmacy & E-commerce**: Online medicine sales, inventory tracking, and prescription management.
- **AI & Analytics**: Predictive health insights and personalized recommendations.

### Other Features
- Multi-tenant support for health facilities
- Role-based access control (global admins, facility admins, healthcare staff)
- Secure SSO integration
- Notifications & reminders (SMS, Email)
- Reporting and analytics dashboards
- Patient account creation and management
- Inventory logs and order tracking
- Discount and coupon management

---

## Tech Stack

- **Backend**: Rust (Actix Web), SeaORM, PostgreSQL
- **Authentication**: Headless SSO integration
- **File Storage**: Multipart file uploads for records, prescriptions, and images
- **Deployment**: Docker, GitHub Actions, optional Tauri for desktop app

---

## Installation

1. Clone the repository:

```bash
git clone https://github.com/CollinsCMK/healthfiti-backend.git
cd healthfiti-backend
````

2. Set up the environment variables:

```bash
# Copy the example file
cp .env.example .env
```

3. Run database migrations:

```bash
chmod +x sea_orm_main.sh up
or
sea-orm-cli migrate up -d ./migration-main
```

4. Start the backend:

```bash
cargo run
```

---

## Usage

* Health facilities can register and onboard their staff.
* Patients can create accounts and book appointments.
* Pharmacies can manage inventory and online orders.
* Admins can monitor system-wide usage and analytics.
