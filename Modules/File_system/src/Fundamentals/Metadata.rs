use Users::{Group_identifier_type, User_identifier_type};

use crate::{Permissions_type, Time_type, Type_type};

use super::Inode_type;

/// File attributes.
///
/// The attributes are metadata associated with the file that stores:
/// - The file type.
/// - The file creation time.
/// - The file modification time.
/// - The file access time.
/// - The file permissions.
/// - The file owner.
/// - The file group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata_type {
    /// The file inode.
    Inode: Option<Inode_type>,
    /// The file type.
    Type: Type_type,
    /// The file creation time.
    Creation_time: Time_type,
    /// The file modification time.
    Modification_time: Time_type,
    /// The file access time.
    Access_time: Time_type,
    /// The file permissions.
    Permissions: Permissions_type,
    /// The file owner.
    User: User_identifier_type,
    /// The file group.
    Group: Group_identifier_type,
}

impl Metadata_type {
    pub const Identifier: u8 = 0x01;

    pub fn Get_default(
        Type: Type_type,
        Current_time: Time_type,
        User: User_identifier_type,
        Group: Group_identifier_type,
    ) -> Option<Self> {
        let Permissions = Permissions_type::New_default(Type);

        Some(Metadata_type {
            Inode: None,
            Type,
            Creation_time: Current_time,
            Modification_time: Current_time,
            Access_time: Current_time,
            Permissions,
            User,
            Group,
        })
    }

    pub fn Get_inode(&self) -> Option<Inode_type> {
        self.Inode
    }

    pub fn Get_type(&self) -> Type_type {
        self.Type
    }

    pub fn Get_creation_time(&self) -> Time_type {
        self.Creation_time
    }

    pub fn Get_modification_time(&self) -> Time_type {
        self.Modification_time
    }

    pub fn Get_access_time(&self) -> Time_type {
        self.Access_time
    }

    pub fn Get_permissions(&self) -> Permissions_type {
        self.Permissions
    }

    pub fn Get_user(&self) -> User_identifier_type {
        self.User
    }

    pub fn Get_group(&self) -> Group_identifier_type {
        self.Group
    }

    pub fn Set_inode(&mut self, Inode: Inode_type) {
        self.Inode = Some(Inode);
    }

    pub fn Set_type(&mut self, Type: Type_type) {
        self.Type = Type;
    }

    pub fn Set_creation_time(&mut self, Time: Time_type) {
        self.Creation_time = Time;
    }

    pub fn Set_modification_time(&mut self, Time: Time_type) {
        self.Modification_time = Time;
    }

    pub fn Set_access_time(&mut self, Time: Time_type) {
        self.Access_time = Time;
    }

    pub fn Set_permissions(&mut self, Permissions: Permissions_type) {
        self.Permissions = Permissions;
    }

    pub fn Set_owner(&mut self, Owner: User_identifier_type) {
        self.User = Owner;
    }

    pub fn Set_group(&mut self, Group: Group_identifier_type) {
        self.Group = Group;
    }
}
