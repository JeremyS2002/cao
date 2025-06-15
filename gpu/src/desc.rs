
// use std::collections::*;

pub trait DescType {
	type InfoType;

	fn to_info(&self) -> Self::InfoType;
}

macro_rules! impl_desc_type_primative {
    ($($name:ident,)*) => {
    	$(
	        impl DescType for $name {
	        	type InfoType = $name;

	        	fn to_info(&self) -> Self::InfoType {
	        		*self
	        	}
	        }
		)*
    };
}

pub(crate) use impl_desc_type_primative;

impl_desc_type_primative!(
	u8, 
	u16, 
	u32, 
	u64, 
	u128, 
	i8, 
	i16, 
	i32, 
	i64, 
	bool,
);

impl<'a, T: DescType> DescType for &'a T {
	type InfoType = T::InfoType;

	fn to_info(&self) -> Self::InfoType {
		(*self).to_info()
	}
}

impl<'a, T: DescType> DescType for &'a [T] {
	type InfoType = Vec<T::InfoType>;

	fn to_info(&self) -> Self::InfoType {
		self.iter().map(|x| x.to_info()).collect()
	}
}

impl<'a> DescType for &'a str {
	type InfoType = String;

	fn to_info(&self) -> Self::InfoType {
		self.to_string()
	}
}

impl<T: DescType> DescType for Vec<T> {
	type InfoType = Vec<T::InfoType>;

	fn to_info(&self) -> Self::InfoType {
		self.iter().map(|x| x.to_info()).collect()
	}
}

// impl<K: DescType + Eq, V: DescType> DescType for HashMap<K, V> {
// 	type InfoType = HashMap<K::InfoType, V::InfoType>;

// 	fn to_info(&self) -> Self::InfoType {
// 		self.iter().map(|(k, v)| (k.to_info(), v.to_info())).collect()
// 	}
// }

macro_rules! imple_desc_type_tuple {
    ($($name:ident,)* ; $($idx:tt,)*) => {
    	impl<$($name : DescType,)*> DescType for ($($name,)*) {
    		type InfoType = ($($name::InfoType,)*);

    		fn to_info(&self) -> Self::InfoType {
    			(
    				$(
    					self.$idx.to_info(),
					)*
				)
    		}
    	}
    };
}

// Call it for tuple sizes 1 to 12
imple_desc_type_tuple!(A, B,; 0, 1,);
imple_desc_type_tuple!(A, B, C,; 0, 1, 2,);
imple_desc_type_tuple!(A, B, C, D,; 0, 1, 2, 3,);
imple_desc_type_tuple!(A, B, C, D, E,; 0, 1, 2, 3, 4,);
imple_desc_type_tuple!(A, B, C, D, E, F,; 0, 1, 2, 3, 4, 5,);
imple_desc_type_tuple!(A, B, C, D, E, F, G,; 0, 1, 2, 3, 4, 5, 6,);
imple_desc_type_tuple!(A, B, C, D, E, F, G, H,; 0, 1, 2, 3, 4, 5, 6, 7,);
imple_desc_type_tuple!(A, B, C, D, E, F, G, H, I,; 0, 1, 2, 3, 4, 5, 6, 7, 8,);
imple_desc_type_tuple!(A, B, C, D, E, F, G, H, I, J,; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,);
imple_desc_type_tuple!(A, B, C, D, E, F, G, H, I, J, K,; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,);
imple_desc_type_tuple!(A, B, C, D, E, F, G, H, I, J, K, L,; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,);
