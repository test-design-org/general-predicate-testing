using System;
using TestingBackend;

namespace backend.DTO
{

    public interface IInput
    {
        Expressions Expression { get; }
        bool IntersectsWith(IInput other);
        IInput Intersect(IInput other);
    }   
}

