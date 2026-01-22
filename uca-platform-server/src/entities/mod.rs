//! SeaORM 实体定义。

pub mod devices;
pub mod passkeys;
pub mod password_policies;
pub mod recovery_codes;
pub mod sessions;
pub mod totp_secrets;
pub mod users;
pub mod students;
pub mod volunteer_records;
pub mod contest_records;
pub mod attachments;
pub mod auth_resets;
pub mod competition_library;
pub mod review_signatures;
pub mod form_fields;
pub mod form_field_values;
pub mod invites;

pub use devices::Entity as Device;
pub use passkeys::Entity as Passkey;
pub use password_policies::Entity as PasswordPolicy;
pub use recovery_codes::Entity as RecoveryCode;
pub use sessions::Entity as Session;
pub use totp_secrets::Entity as TotpSecret;
pub use users::Entity as User;
pub use students::Entity as Student;
pub use volunteer_records::Entity as VolunteerRecord;
pub use contest_records::Entity as ContestRecord;
pub use attachments::Entity as Attachment;
pub use auth_resets::Entity as AuthReset;
pub use competition_library::Entity as CompetitionLibrary;
pub use review_signatures::Entity as ReviewSignature;
pub use form_fields::Entity as FormField;
pub use form_field_values::Entity as FormFieldValue;
pub use invites::Entity as Invite;
