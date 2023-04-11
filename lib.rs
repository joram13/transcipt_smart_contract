#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod transcipt {
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    /// Specify Transcipt error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidInput,
        AccessNotAllowed
    }

    /// Specify the Transcipt result type.
    pub type Result<T> = core::result::Result<T, Error>;


    

    /// Create storage for a Transcipt contract.
    #[ink(storage)]
    pub struct Transcipt{
        
        //for each student define a list of people allowed to access the grade 
        accessstudents: Mapping<AccountId, Vec<AccountId>>,
        //store students, teachers, admins, and classes in lists 
        students: Vec<AccountId>,
        teachers: Vec<AccountId>,
        admins: Vec<AccountId>,
        class_list: Vec<String>,
        //store a mapping from stduent and class to a vector of the students grades in that class
        grades: Mapping<(AccountId, String), Vec<u8>>,
        //store a mapping from a class to the teacher and a vector of students in that class
        classes: Mapping<String,( AccountId, Vec<AccountId>)>,
    }

    impl Transcipt {
        /// Create a new Transcipt contract with the caller as first admin.
        #[ink(constructor)]
        pub fn new() -> Self {
            
            //initiate default storage items
            let accessstudents = Mapping::default();
            let teachers = Vec::default();
            let class_list = Vec::default();
            let grades = Mapping::default();
            let students = Vec::default();
            let classes = Mapping::default(); 
            let mut admins = Vec::default(); 
            
            //add contract caller as admin
            admins.push(Self::env().caller());

            Self {
                accessstudents,
                students,
                teachers,
                grades,
                classes,
                admins,
                class_list
            }
            
        }
        

        //adds teacher to storage 
        #[ink(message)]
        pub fn add_teacher(&mut self, teacher_id: AccountId) -> Result<()>{
            //only the admin has access
            if self.admins.contains(&Self::env().caller()) {
                // only new teachers can be added 
                if !self.teachers.contains(&teacher_id) {
                    //adding teacher
                    self.teachers.push(teacher_id);
                    Ok(())
            } else {
                Err(Error::InvalidInput)
            }
            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }

        //adding students to the system
        #[ink(message)]
        pub fn add_student(&mut self, student_id: AccountId) -> Result<()>{
            //only admin has access
            if self.admins.contains(&Self::env().caller()) {
                //only new students can be added
                if !self.students.contains(&student_id) {
                    //add students and initate access list with student in it 
                    self.students.push(student_id);
                    self.accessstudents.insert(student_id, &[student_id].to_vec());
                    Ok(())
                } else {
                    Err(Error::InvalidInput)
                }
            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }

        //add admins 
        #[ink(message)]
        pub fn add_admins(&mut self, admin_id: AccountId) -> Result<()>{
            //only admins can access
            if self.admins.contains(&Self::env().caller()) {
                //only new admins can be added
                if !self.admins.contains(&admin_id) {
                    self.admins.push(admin_id);
                    Ok(())
                } else {
                    Err(Error::InvalidInput)
                }
            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }

        //adding classes to the system
        #[ink(message)]
        pub fn add_classes(&mut self,class_name: String, teacher_id: AccountId, student_ids: Vec<AccountId>) -> Result<()>{
            //only admins have access
            if self.admins.contains(&Self::env().caller()) {
                //teacher must be saved as teacher, students must be saved as students, the clast must be new 
                if self.teachers.contains(&teacher_id) && student_ids.iter().all(|x| self.students.contains(x)) && !self.class_list.contains(&class_name) {
                    //adding the class to the list of classes and save students and teacher in mapping
                    self.classes.insert(&class_name, &(teacher_id, student_ids));
                    self.class_list.push(class_name);
                    Ok(())
                } else {
                    Err(Error::InvalidInput)
                }
            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }

        //adding a score to a student in a class
        #[ink(message)]
        pub fn add_score(&mut self,class_name: String, student_id: AccountId, grade: u8) -> Result<()>{

            //accessing class info
            let class_info = if let Some(class_info) = self.classes.get(&class_name) { class_info } else { return Err(Error::InvalidInput)  };
            let teacher = class_info.0;
            let students = class_info.1;

            //only teacher of the class can add and student must be stored as one 
            if teacher == Self::env().caller() && students.contains(&student_id) {
                //add grade to list of grades of student in that class
                let mut current_grades = if let Some(current_grades) = self.grades.get((student_id, &class_name)) { current_grades } else { [].to_vec() };
                current_grades.push(grade);
                self.grades.insert((student_id, &class_name), &current_grades);
                Ok(())

            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }
    

        //adding any account to be able to access the grades of a specific student
        #[ink(message)]
        pub fn add_accessstudents(&mut self, student_id: AccountId, new_access_id: AccountId) -> Result<()> {
            //only admins, teachers or the specific student specified in the input can change this 
            if self.teachers.contains(&Self::env().caller()) || self.admins.contains(&Self::env().caller()) || Self::env().caller() == student_id {
                //must be a new acount id 
                if !self.accessstudents.get(student_id).unwrap().contains(&new_access_id) {
                    //add new id to list 
                    let mut current_access = self.accessstudents.get(student_id).unwrap_or_default();
                    current_access.push(new_access_id);
                    self.accessstudents.insert(student_id, &current_access);
                    
                    Ok(())
                } else {
                    Err(Error::InvalidInput)
                }
            } else {
                return Err(Error::AccessNotAllowed) 
            }
            
        }

        //access the grades of a student for a specific class
        #[ink(message)]
        pub fn access_grades(&self,class_name: String, student_id: AccountId) -> Result<Vec<u8>> {
            //get all people who have access to the grades of the student
            let has_access = self.accessstudents.get(student_id).unwrap_or_default();
            //admins, teachers, and people on the allow list have access
            if self.teachers.contains(&Self::env().caller()) || self.admins.contains(&Self::env().caller()) || has_access.contains(&Self::env().caller()) {
                //get and return grades
                let current_grades = self.grades.get((student_id, &class_name)).unwrap_or_default();
                return Ok(current_grades)
            } else {
                Err(Error::AccessNotAllowed) 
            }
            
        }

        //remove a person from the access list of a student
        #[ink(message)]
        pub fn remove_accessstudents(&mut self, student_id: AccountId, remove_access_id: AccountId) -> Result<()> {
            
            if self.teachers.contains(&Self::env().caller()) || self.admins.contains(&Self::env().caller()) {

                let mut current_access = self.accessstudents.get(student_id).unwrap_or_default();
                if let Some(index) = current_access.iter().position(|x| *x == remove_access_id) {
                    current_access.remove(index);
                }

                self.accessstudents.insert(student_id, &current_access);
                
                Ok(())
            } else {
                return Err(Error::AccessNotAllowed) 
            }
            
        }

        #[ink(message)]
        pub fn remove_admins(&mut self, admin_id: AccountId) -> Result<()>{

            if self.admins.contains(&Self::env().caller()) && self.admins.len() >= 2 {
                if let Some(index) = self.admins.iter().position(|x| *x == admin_id) {
                    self.admins.remove(index);
                }
                Ok(())
            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }

        #[ink(message)]
        pub fn remove_classes(&mut self,class_name: String) -> Result<()>{



            if self.admins.contains(&Self::env().caller()) {

                let class_info = self.classes.get(&class_name).unwrap();
                let students = class_info.1;

                for student in students.iter() {
                    self.grades.take((student, &class_name));
                }

                self.classes.take(&class_name);
                

                if let Some(index) = self.class_list.iter().position(|x| *x == class_name) {
                    self.class_list.remove(index);
                }

                Ok(())

            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }

        #[ink(message)]
        pub fn unenroll_student(&mut self,class_name: String, student_id: AccountId) -> Result<()>{
            if self.admins.contains(&Self::env().caller()) {

                let class_info = self.classes.get(&class_name).unwrap();
                let mut students = class_info.1;
                

                if self.students.contains(&student_id) && students.contains(&student_id) {

                    if let Some(index) = students.iter().position(|x| *x == student_id) {
                        students.remove(index);
                    }

                    self.classes.insert(&class_name, &(class_info.0, students));

                    self.grades.take((&student_id, &class_name));
                    Ok(())
                } else {{
                    return Err(Error::InvalidInput) 
                }}

            } else {
                return Err(Error::AccessNotAllowed) 
            }


        }

        #[ink(message)]
        pub fn enroll_student(&mut self,class_name: String, student_id: AccountId) -> Result<()>{
            if self.admins.contains(&Self::env().caller()) {

                let class_info = self.classes.get(&class_name).unwrap();
                let mut students = class_info.1;

                if self.students.contains(&student_id) && !students.contains(&student_id) {
                    
                    students.push(student_id);

                    self.classes.insert(&class_name, &(class_info.0, students));

                    self.grades.insert((&student_id, &class_name), &Vec::<u8>::new());

                    Ok(())
                } else {{
                    return Err(Error::InvalidInput) 
                }}

            } else {
                return Err(Error::AccessNotAllowed) 
            }

        }


        #[ink(message)]
        pub fn change_teacher(&mut self,class_name: String, teacher_id: AccountId) -> Result<()>{

            if self.admins.contains(&Self::env().caller()) {

                

                if self.teachers.contains(&teacher_id)  {

                    let class_info = self.classes.get(&class_name).unwrap();
                    let students = class_info.1;
                    

                    self.classes.insert(&class_name, &(teacher_id, students));

                    Ok(())
                } else {
                    return Err(Error::InvalidInput) 
                }

            } else {
                return Err(Error::AccessNotAllowed) 
            }

        }

        #[ink(message)]
        pub fn remove_teacher(&mut self, teacher_id: AccountId) -> Result<()>{

            if self.admins.contains(&Self::env().caller()) {
                if self.teachers.contains(&teacher_id) {

                    if let Some(index) = self.teachers.iter().position(|x| *x == teacher_id) {
                        self.teachers.remove(index);
                    }

                    Ok(())
                } else {
                    return Err(Error::InvalidInput) 
                }
            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }


        #[ink(message)]
        pub fn remove_student(&mut self, student_id: AccountId) -> Result<()>{

            if self.admins.contains(&Self::env().caller()) {
                if self.students.contains(&student_id) {

                    let mut student_classes = Vec::<String>::new();
                    for class in self.class_list.iter() {
                        if self.classes.get(class).unwrap().1.contains(&student_id) {
                            //self.unenroll_student(class, student_id);
                            student_classes.push((&class).to_string());
                            //self.grades.take((student_id, &class));
                        }
                    }

                    

                    for class in student_classes.iter() {
                        self.grades.take((student_id, &class));
                        //self.unenroll_student((&class).to_string(), student_id);
                        match self.unenroll_student((&class).to_string(), student_id) {
                            Ok(_) => {
                                continue
                            }
                            Err(e) => {
                                return Err(e)
                            }
                        }
                    }

                    if let Some(index) = self.students.iter().position(|x| *x == student_id) {
                        self.students.remove(index);
                    }

                    


                    Ok(())
                } else {
                    return Err(Error::InvalidInput) 
                }
            } else {
                return Err(Error::AccessNotAllowed) 
            }
        }

        

    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // We define some helper Accounts to make our tests more readable
        fn default_accounts() -> ink::env::test::DefaultAccounts<Environment> {
            ink::env::test::default_accounts::<Environment>()
        }


        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        fn charlie() -> AccountId {
            default_accounts().charlie
        }

        fn eve() -> AccountId {
            default_accounts().eve
        }

        fn frank() -> AccountId {
            default_accounts().frank
        }


        #[ink::test]
        fn new_works() {
            let contract = Transcipt::new();
            assert_eq!(contract.admins, [alice()] );
        }

        #[ink::test]
        fn add_teacher_works() {
            
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(bob()).is_ok());
            assert!(contract.add_teacher(bob()).is_err());
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.add_teacher(eve()).is_err());
            assert_eq!(contract.teachers, [bob()] );
            
            
        }

        #[ink::test]
        fn add_students_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_student(bob()).is_err());
            assert_eq!(contract.students, [bob()] );
            assert_eq!(contract.accessstudents.get(bob()), Some([bob()].to_vec()) );
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.add_student(eve()).is_err());

        }

        #[ink::test]
        fn add_admins_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_admins(bob()).is_ok());
            assert!(contract.add_admins(bob()).is_err());
            assert_eq!(contract.admins, [alice(), bob()] );
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.add_admins(frank()).is_err());
        }

        #[ink::test]
        fn add_classes_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_err());
            assert!(contract.add_classes("CS51".to_string(),alice(), [eve()].to_vec()).is_err());
            assert!(contract.add_classes("CS51".to_string(),eve(), [bob()].to_vec()).is_err());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [bob()].to_vec())));
            assert!(contract.class_list.contains(&"CS50".to_string()));
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.add_classes("CS51".to_string(),alice(), [bob()].to_vec()).is_err());

        }

        #[ink::test]
        fn add_score_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_teacher(eve()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_ok());
            assert!(contract.add_classes("CS51".to_string(),eve(), [bob()].to_vec()).is_ok());
            assert!(contract.add_score("CS50".to_string(), bob(), 2).is_ok());
            assert_eq!(contract.grades.get((bob(), "CS50".to_string())),Some([2].to_vec()));
            assert_eq!(contract.access_grades("CS50".to_string(), bob()).unwrap(),[2].to_vec());
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.add_score("CS50".to_string(), bob(), 3).is_ok());
            assert_eq!(contract.access_grades("CS50".to_string(), bob()).unwrap(),[2,3].to_vec());
            assert!(contract.add_score("CS51".to_string(), bob(), 3).is_err());
        }


        #[ink::test]
        fn add_accessstudents_works_1() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_teacher(eve()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_ok());
            assert!(contract.add_score("CS50".to_string(), bob(), 2).is_ok());
            assert_eq!(contract.accessstudents.get(bob()).unwrap(), [bob()].to_vec());
            assert!(contract.add_accessstudents(bob(), frank()).is_ok());
            assert_eq!(contract.accessstudents.get(bob()).unwrap(), [bob(), frank()].to_vec());
            assert!(contract.add_accessstudents(bob(), frank()).is_err());
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.add_accessstudents(bob(), charlie()).is_ok());

            
        }

        #[ink::test]
        fn add_accessstudents_works_2() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_teacher(eve()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),eve(), [bob()].to_vec()).is_ok());
            assert_eq!(contract.accessstudents.get(bob()).unwrap(), [bob()].to_vec());
            assert!(contract.add_accessstudents(bob(), frank()).is_ok());
            assert_eq!(contract.accessstudents.get(bob()).unwrap(), [bob(), frank()].to_vec());
            assert!(contract.add_accessstudents(bob(), frank()).is_err());
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_teacher(alice()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.add_accessstudents(bob(), charlie()).is_err());

            
        }

        #[ink::test]
        fn add_access_grades_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_student(alice()).is_ok());
            assert!(contract.add_teacher(eve()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_ok());
            assert!(contract.add_classes("CS51".to_string(),eve(), [alice()].to_vec()).is_ok());
            assert!(contract.add_classes("CS52".to_string(),eve(), [bob()].to_vec()).is_ok());
            assert!(contract.add_score("CS50".to_string(), bob(), 2).is_ok());
            assert_eq!(contract.access_grades("CS50".to_string(), bob()).unwrap(),[2].to_vec());
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_teacher(alice()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert_eq!(contract.access_grades("CS51".to_string(), alice()).unwrap(),[].to_vec());
            assert!(contract.access_grades("CS52".to_string(), bob()).is_err());
            
        }

        #[ink::test]
        fn remove_access_grades_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_student(alice()).is_ok());
            assert!(contract.add_teacher(eve()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS51".to_string(),eve(), [alice()].to_vec()).is_ok());
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_teacher(alice()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert_eq!(contract.access_grades("CS51".to_string(), alice()).unwrap(),[].to_vec());
            assert!(contract.remove_accessstudents(alice(), alice()).is_err());
            assert!(contract.access_grades("CS51".to_string(), alice()).is_ok());
            
        } 

        #[ink::test]
        fn remove_classes_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_student(eve()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_ok());
            assert!(contract.add_classes("CS51".to_string(),alice(), [eve()].to_vec()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [bob()].to_vec())));
            assert!(contract.class_list.contains(&"CS50".to_string()));
            assert!(contract.remove_classes("CS50".to_string()).is_ok());
            assert!(!contract.class_list.contains(&"CS50".to_string()));
            assert!(contract.class_list.contains(&"CS51".to_string()));
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.remove_classes("CS51".to_string()).is_err());

        }

        #[ink::test]
        fn enroll_unenroll_student_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_student(eve()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [bob()].to_vec())));
            assert!(contract.enroll_student("CS50".to_string(), eve()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [bob(), eve()].to_vec())));
            assert!(contract.enroll_student("CS50".to_string(), eve()).is_err());
            assert!(contract.enroll_student("CS50".to_string(), charlie()).is_err());
            assert!(contract.unenroll_student("CS50".to_string(), eve()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [bob()].to_vec())));
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.enroll_student("CS50".to_string(), eve()).is_err());

        }

        #[ink::test]
        fn change_teacher_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_teacher(eve()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob()].to_vec()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [bob()].to_vec())));
            assert!(contract.change_teacher("CS50".to_string(), eve()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((eve(), [bob()].to_vec())));
            assert!(contract.change_teacher("CS50".to_string(), charlie()).is_err());
            assert!(contract.change_teacher("CS50".to_string(), bob()).is_err());

            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.change_teacher("CS50".to_string(), alice()).is_err());

        }

        #[ink::test]
        fn remove_teacher_works() {
            
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(bob()).is_ok());
            assert!(contract.add_teacher(charlie()).is_ok());
            assert_eq!(contract.teachers, [bob(), charlie()] );
            assert!(contract.remove_teacher(charlie()).is_ok());
            assert_eq!(contract.teachers, [bob()] );
            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.remove_teacher(charlie()).is_err());
            
            
        }

        #[ink::test]
        fn remove_student_works() {
            let mut contract = Transcipt::new();
            assert!(contract.add_teacher(alice()).is_ok());
            assert!(contract.add_student(eve()).is_ok());
            assert!(contract.add_student(bob()).is_ok());
            assert!(contract.add_classes("CS50".to_string(),alice(), [bob(), eve()].to_vec()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [bob(), eve()].to_vec())));
          
            assert!(contract.add_score("CS50".to_string(), bob(), 2).is_ok());
            assert!(contract.add_score("CS50".to_string(), eve(), 3).is_ok());
            assert_eq!(contract.grades.get((bob(), "CS50".to_string())),Some([2].to_vec()));
            assert_eq!(contract.grades.get((eve(), "CS50".to_string())),Some([3].to_vec()));

            assert!(contract.remove_student(bob()).is_ok());
            assert_eq!(contract.classes.get("CS50".to_string()),Some((alice(), [eve()].to_vec())));
            assert_eq!(contract.grades.get((bob(), "CS50".to_string())), None);
            assert_eq!(contract.grades.get((eve(), "CS50".to_string())),Some([3].to_vec()));
            assert!(!contract.students.contains(&bob()));

                        

            assert!(contract.add_admins(charlie()).is_ok());
            assert!(contract.remove_admins(alice()).is_ok());
            assert!(contract.remove_student(eve()).is_err());

        }



// remove student



        
        

    }
}


