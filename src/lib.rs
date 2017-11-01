extern crate glium;

use glium::vertex::{Vertex, VertexBuffer, IntoVerticesSource, VerticesSource};
use std::any::TypeId;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::{mem, ptr};

/// A type-erased `VertexBuffer` which can be safely borrowed as the original `VertexBuffer<T>`
/// using a dynamic type check.
pub struct TypedVertexBufferAny {
    type_id: TypeId,
    buffer: VertexBuffer<u8>,
}

impl TypedVertexBufferAny {
    /// Borrows buffer immutably as typed, checking the type at runtime.
    ///
    /// Returns an error if the type is not the one given on construction.
    pub fn downcast<T: Vertex + 'static>(&self) -> Result<&VertexBuffer<T>, IncorrectTypeError> {
        if self.type_id == TypeId::of::<T>() {
            Ok(unsafe {
                &*(&self.buffer as *const _ as *const VertexBuffer<T>)
            })
        } else {
            Err(IncorrectTypeError)
        }
    }

    /// Borrows the buffer mutably as typed, checking the type at runtime.
    ///
    /// Returns an error if the type is not the one given on construction.
    pub fn downcast_mut<T: Vertex + 'static>(
        &mut self,
    ) -> Result<&mut VertexBuffer<T>, IncorrectTypeError> {
        if self.type_id == TypeId::of::<T>() {
            Ok(unsafe {
                &mut *(&mut self.buffer as *mut _ as *mut VertexBuffer<T>)
            })
        } else {
            Err(IncorrectTypeError)
        }
    }
}

impl<T> From<VertexBuffer<T>> for TypedVertexBufferAny
where
    T: Vertex + 'static,
{
    fn from(buffer: VertexBuffer<T>) -> TypedVertexBufferAny {
        // These should statically be true, since `T` is stored by glium only via a zero-sized
        // `PhantomData`.
        assert_eq!(mem::size_of_val(&buffer), mem::size_of::<VertexBuffer<T>>());
        assert_eq!(
            mem::align_of_val(&buffer),
            mem::align_of::<VertexBuffer<T>>()
        );

        let type_erased = TypedVertexBufferAny {
            type_id: TypeId::of::<T>(),
            buffer: unsafe { ptr::read(&buffer as *const _ as *const VertexBuffer<u8>) },
        };
        mem::forget(buffer);

        type_erased
    }
}

impl<'a> IntoVerticesSource<'a> for &'a TypedVertexBufferAny {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        self.buffer.into_vertices_source()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct IncorrectTypeError;

impl Error for IncorrectTypeError {
    fn description(&self) -> &str {
        "incorrect buffer type"
    }
}

impl Display for IncorrectTypeError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{}", self.description())
    }
}
